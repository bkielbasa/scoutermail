package main

import (
	"context"
	"encoding/base64"
	"fmt"
	"io"
	"mime"
	"mime/multipart"
	"net/mail"
	"regexp"
	"strings"

	"github.com/emersion/go-imap"
	"github.com/emersion/go-imap/client"
)

type Email struct {
	ID              uint32 `json:"id"`
	From            string `json:"from"`
	Subject         string `json:"subject"`
	Date            string `json:"date"`
	Snippet         string `json:"snippet"`
	AttachmentCount int    `json:"attachmentCount"`
	Read            bool   `json:"read"`
}

type EmailPage struct {
	Emails     []Email `json:"emails"`
	TotalCount uint32  `json:"totalCount"`
}

type EmailContent struct {
	Headers     map[string]string `json:"headers"`
	TextBody    string            `json:"textBody"`
	HTMLBody    string            `json:"htmlBody"`
	Attachments []Attachment      `json:"attachments"`
}

type Attachment struct {
	Filename    string `json:"filename"`
	ContentType string `json:"contentType"`
	Size        int    `json:"size"`
	Data        string `json:"data"` // Base64 encoded
}

// countAttachments recursively counts attachments in a message body structure
func countAttachments(body *imap.BodyStructure) int {
	if body == nil {
		return 0
	}

	count := 0

	// Only count as attachment if it has explicit attachment disposition
	if body.Disposition == "attachment" {
		fmt.Printf("DEBUG: Found attachment - Disposition: %s, MIMEType: %s, Filename: %s\n",
			body.Disposition, body.MIMEType, body.DispositionParams["filename"])
		count++
	}

	// Recursively check child parts
	if body.Parts != nil {
		for _, part := range body.Parts {
			count += countAttachments(part)
		}
	}

	return count
}

func FetchEmails(ctx context.Context, page int, pageSize int, account *Account, password string) (EmailPage, error) {
	fmt.Printf("DEBUG: FetchEmails called with page=%d, pageSize=%d\n", page, pageSize)

	// Connect to server
	var c *client.Client
	var err error
	if account.UseSSL {
		c, err = client.DialTLS(fmt.Sprintf("%s:%d", account.EmailServer, account.Port), nil)
	} else {
		c, err = client.Dial(fmt.Sprintf("%s:%d", account.EmailServer, account.Port))
	}
	if err != nil {
		return EmailPage{}, err
	}
	defer c.Logout()

	// Login
	if err := c.Login(account.Username, password); err != nil {
		return EmailPage{}, err
	}

	// Select INBOX
	mbox, err := c.Select("INBOX", false)
	if err != nil {
		return EmailPage{}, err
	}
	total := mbox.Messages
	fmt.Printf("DEBUG: Total messages in mailbox: %d\n", total)

	if total == 0 {
		return EmailPage{Emails: []Email{}, TotalCount: 0}, nil
	}

	// Calculate which messages to fetch (most recent first)
	var from, to uint32
	startIndex := uint32((page - 1) * pageSize)

	if startIndex >= total {
		fmt.Printf("DEBUG: startIndex (%d) >= total (%d), returning empty result\n", startIndex, total)
		return EmailPage{Emails: []Email{}, TotalCount: total}, nil
	}

	// Calculate the range (most recent messages first, so we count from the end)
	to = total - startIndex
	from = to - uint32(pageSize) + 1
	if from < 1 {
		from = 1
	}

	fmt.Printf("DEBUG: Fetching messages from %d to %d (page %d)\n", from, to, page)

	seqset := new(imap.SeqSet)
	seqset.AddRange(from, to)

	messages := make(chan *imap.Message, pageSize)
	done := make(chan error, 1)
	go func() {
		done <- c.Fetch(seqset, []imap.FetchItem{imap.FetchEnvelope, imap.FetchInternalDate, imap.FetchUid, imap.FetchFlags, imap.FetchBodyStructure}, messages)
	}()

	var emails []Email
	for msg := range messages {
		// Count attachments from body structure
		attachmentCount := 0
		if msg.BodyStructure != nil {
			attachmentCount = countAttachments(msg.BodyStructure)
		}

		// Check if email is read based on IMAP flags
		isRead := false
		for _, flag := range msg.Flags {
			if flag == imap.SeenFlag {
				isRead = true
				break
			}
		}

		// Generate a simple snippet from subject or use a placeholder
		snippet := generateSimpleSnippet(msg)

		email := Email{
			ID:              msg.Uid,
			From:            msg.Envelope.From[0].MailboxName + "@" + msg.Envelope.From[0].HostName,
			Subject:         msg.Envelope.Subject,
			Date:            msg.Envelope.Date.Format("2006-01-02"),
			Snippet:         snippet,
			AttachmentCount: attachmentCount,
			Read:            isRead,
		}
		emails = append([]Email{email}, emails...) // Prepend to reverse order (most recent first)
	}
	if err := <-done; err != nil {
		return EmailPage{}, err
	}

	fmt.Printf("DEBUG: Returning %d emails for page %d\n", len(emails), page)
	return EmailPage{Emails: emails, TotalCount: total}, nil
}

// generateSimpleSnippet creates a simple snippet without fetching full content
func generateSimpleSnippet(msg *imap.Message) string {
	// Use subject as snippet, or a simple placeholder
	if msg.Envelope != nil && msg.Envelope.Subject != "" {
		return msg.Envelope.Subject
	}
	return "No subject"
}

// generateSnippet extracts a snippet from the email content
func generateSnippet(c *client.Client, msg *imap.Message) string {
	fmt.Printf("DEBUG: Generating snippet for email UID %d\n", msg.Uid)

	if msg.BodyStructure == nil {
		fmt.Printf("DEBUG: No body structure for email UID %d\n", msg.Uid)
		return ""
	}

	// Try to get a small portion of the email body for snippet
	seqset := new(imap.SeqSet)
	seqset.AddNum(msg.Uid)

	// Try to get text content from the body structure
	content := extractSnippetFromBodyStructure(c, seqset, msg.BodyStructure)
	if content != "" {
		fmt.Printf("DEBUG: Generated snippet for UID %d: %s\n", msg.Uid, content[:min(50, len(content))])
		return content
	}

	fmt.Printf("DEBUG: No snippet generated from body structure for UID %d, trying raw body\n", msg.Uid)

	// Fallback: try to get the full body and extract snippet
	section := &imap.BodySectionName{}
	messages := make(chan *imap.Message, 1)
	done := make(chan error, 1)

	go func() {
		done <- c.UidFetch(seqset, []imap.FetchItem{section.FetchItem()}, messages)
	}()

	for msg := range messages {
		if msg == nil {
			continue
		}
		if r := msg.GetBody(section); r != nil {
			b, err := io.ReadAll(r)
			if err == nil && len(b) > 0 {
				rawBody := string(b)
				content := extractSnippetFromRaw(rawBody)
				if content != "" {
					fmt.Printf("DEBUG: Generated snippet from raw body for UID %d: %s\n", msg.Uid, content[:min(50, len(content))])
				} else {
					fmt.Printf("DEBUG: No snippet generated from raw body for UID %d\n", msg.Uid)
				}
				<-done
				return content
			}
		}
	}
	<-done

	fmt.Printf("DEBUG: Failed to generate snippet for UID %d\n", msg.Uid)
	return ""
}

// extractSnippetFromBodyStructure tries to extract snippet from body structure
func extractSnippetFromBodyStructure(c *client.Client, seqset *imap.SeqSet, body *imap.BodyStructure) string {
	if body == nil {
		return ""
	}

	fmt.Printf("DEBUG: Extracting snippet from body structure, MIME type: %s\n", body.MIMEType)

	// Handle multipart messages
	if body.MIMEType == "multipart" && body.Parts != nil {
		fmt.Printf("DEBUG: Processing multipart message with %d parts\n", len(body.Parts))
		for i, part := range body.Parts {
			fmt.Printf("DEBUG: Part %d MIME type: %s\n", i, part.MIMEType)
			if part.MIMEType == "text/plain" {
				fmt.Printf("DEBUG: Found text/plain part at index %d, extracting content\n", i)
				content := extractTextPart(c, seqset, part, "")
				if content != "" {
					fmt.Printf("DEBUG: Successfully extracted text/plain content: %s\n", content[:min(50, len(content))])
					return extractSnippetFromText(content)
				}
			}
		}
		// If no text/plain, try text/html
		for i, part := range body.Parts {
			if part.MIMEType == "text/html" {
				fmt.Printf("DEBUG: Found text/html part at index %d, extracting content\n", i)
				content := extractTextPart(c, seqset, part, "")
				if content != "" {
					fmt.Printf("DEBUG: Successfully extracted text/html content: %s\n", content[:min(50, len(content))])
					return extractSnippetFromHTML(content)
				}
			}
		}
	} else if body.MIMEType == "text/plain" {
		fmt.Printf("DEBUG: Processing text/plain message\n")
		content := extractTextPart(c, seqset, body, "")
		if content != "" {
			fmt.Printf("DEBUG: Successfully extracted text/plain content: %s\n", content[:min(50, len(content))])
			return extractSnippetFromText(content)
		}
	} else if body.MIMEType == "text/html" {
		fmt.Printf("DEBUG: Processing text/html message\n")
		content := extractTextPart(c, seqset, body, "")
		if content != "" {
			fmt.Printf("DEBUG: Successfully extracted text/html content: %s\n", content[:min(50, len(content))])
			return extractSnippetFromHTML(content)
		}
	}

	fmt.Printf("DEBUG: No suitable content found in body structure\n")
	return ""
}

// extractTextPart extracts text content from a specific part
func extractTextPart(c *client.Client, seqset *imap.SeqSet, body *imap.BodyStructure, path string) string {
	section := &imap.BodySectionName{}
	if path != "" {
		section.Specifier = imap.PartSpecifier(path)
	}

	messages := make(chan *imap.Message, 1)
	done := make(chan error, 1)
	go func() {
		done <- c.UidFetch(seqset, []imap.FetchItem{section.FetchItem()}, messages)
	}()

	for msg := range messages {
		if msg == nil {
			continue
		}
		if r := msg.GetBody(section); r != nil {
			b, err := io.ReadAll(r)
			if err == nil && len(b) > 0 {
				encoding := body.Encoding
				decoded := decodeTextContent(b, encoding)
				<-done
				return decoded
			}
		}
	}
	<-done
	return ""
}

// extractSnippetFromText extracts a snippet from plain text
func extractSnippetFromText(text string) string {
	// Clean the text
	text = cleanTextContent(text)

	// Remove extra whitespace
	text = strings.Join(strings.Fields(text), " ")

	// Limit to 150 characters
	if len(text) > 150 {
		text = text[:150] + "..."
	}

	return text
}

// extractSnippetFromHTML extracts a snippet from HTML content
func extractSnippetFromHTML(html string) string {
	// Simple HTML tag removal
	html = strings.ReplaceAll(html, "<br>", " ")
	html = strings.ReplaceAll(html, "<br/>", " ")
	html = strings.ReplaceAll(html, "<br />", " ")
	html = strings.ReplaceAll(html, "</p>", " ")
	html = strings.ReplaceAll(html, "</div>", " ")

	// Remove HTML tags
	re := regexp.MustCompile(`<[^>]*>`)
	text := re.ReplaceAllString(html, "")

	return extractSnippetFromText(text)
}

// extractSnippetFromRaw extracts snippet from raw email
func extractSnippetFromRaw(rawEmail string) string {
	// Parse the raw email
	msg, err := mail.ReadMessage(strings.NewReader(rawEmail))
	if err != nil {
		return ""
	}

	// Try to get text content
	mediaType, params, err := mime.ParseMediaType(msg.Header.Get("Content-Type"))
	if err != nil {
		mediaType = "text/plain"
	}

	if strings.HasPrefix(mediaType, "multipart/") {
		boundary := params["boundary"]
		if boundary == "" {
			return ""
		}

		mr := multipart.NewReader(msg.Body, boundary)
		for {
			part, err := mr.NextPart()
			if err != nil {
				break
			}

			if part.Header.Get("Content-Type") == "text/plain" {
				body, err := io.ReadAll(part)
				if err == nil {
					return extractSnippetFromText(string(body))
				}
			}
		}
	} else if mediaType == "text/plain" {
		body, err := io.ReadAll(msg.Body)
		if err == nil {
			return extractSnippetFromText(string(body))
		}
	}

	return ""
}

func FetchFolders(ctx context.Context, account *Account, password string) ([]string, error) {
	var c *client.Client
	var err error
	if account.UseSSL {
		c, err = client.DialTLS(fmt.Sprintf("%s:%d", account.EmailServer, account.Port), nil)
	} else {
		c, err = client.Dial(fmt.Sprintf("%s:%d", account.EmailServer, account.Port))
	}
	if err != nil {
		return nil, err
	}
	defer c.Logout()

	if err := c.Login(account.Username, password); err != nil {
		return nil, err
	}

	mailboxes := make(chan *imap.MailboxInfo, 10)
	done := make(chan error, 1)
	var folders []string

	go func() {
		done <- c.List("", "*", mailboxes)
	}()

	for {
		select {
		case <-ctx.Done():
			return nil, ctx.Err()
		case mbox, ok := <-mailboxes:
			if !ok {
				return folders, <-done
			}
			folders = append(folders, mbox.Name)
		}
	}
}

func FetchEmailContent(ctx context.Context, uid uint32, account *Account, password string) (EmailContent, error) {
	var c *client.Client
	var err error
	if account.UseSSL {
		c, err = client.DialTLS(fmt.Sprintf("%s:%d", account.EmailServer, account.Port), nil)
	} else {
		c, err = client.Dial(fmt.Sprintf("%s:%d", account.EmailServer, account.Port))
	}
	if err != nil {
		return EmailContent{}, err
	}
	defer c.Logout()

	if err := c.Login(account.Username, password); err != nil {
		return EmailContent{}, err
	}

	_, err = c.Select("INBOX", false)
	if err != nil {
		return EmailContent{}, err
	}

	seqset := new(imap.SeqSet)
	seqset.AddNum(uid)

	// Try multiple approaches to get the email content
	var rawBody string
	var bodyStructure *imap.BodyStructure

	// First, try to get the full message
	messages := make(chan *imap.Message, 1)
	done := make(chan error, 1)

	go func() {
		done <- c.UidFetch(seqset, []imap.FetchItem{imap.FetchBodyStructure}, messages)
	}()

	for msg := range messages {
		if msg == nil {
			continue
		}
		bodyStructure = msg.BodyStructure
	}

	if err := <-done; err != nil {
		return EmailContent{}, err
	}

	// Try to get the body using different approaches
	section := &imap.BodySectionName{}
	messages = make(chan *imap.Message, 1)
	done = make(chan error, 1)

	go func() {
		done <- c.UidFetch(seqset, []imap.FetchItem{section.FetchItem()}, messages)
	}()

	for msg := range messages {
		if msg == nil {
			continue
		}
		if r := msg.GetBody(section); r != nil {
			b, err := io.ReadAll(r)
			if err == nil && len(b) > 0 {
				rawBody = string(b)
				break
			}
		}
	}

	if err := <-done; err != nil {
		return EmailContent{}, err
	}

	if rawBody == "" {
		return EmailContent{}, fmt.Errorf("no email body found")
	}

	// Parse the raw email
	content, err := parseRawEmail(rawBody)
	if err != nil {
		return EmailContent{}, err
	}

	// If we still don't have content, try to extract from body structure
	if content.TextBody == "" && content.HTMLBody == "" && bodyStructure != nil {
		content = extractFromBodyStructure(c, seqset, bodyStructure)
	}

	return content, nil
}

// extractFromBodyStructure extracts content from IMAP body structure
func extractFromBodyStructure(c *client.Client, seqset *imap.SeqSet, body *imap.BodyStructure) EmailContent {
	content := EmailContent{
		Headers:     make(map[string]string),
		TextBody:    "",
		HTMLBody:    "",
		Attachments: []Attachment{},
	}

	// Recursively extract content from body structure
	extractParts(c, seqset, body, "", &content)

	return content
}

// extractParts recursively extracts content from body structure parts
func extractParts(c *client.Client, seqset *imap.SeqSet, body *imap.BodyStructure, path string, content *EmailContent) {
	if body == nil {
		return
	}

	// Handle multipart messages
	if strings.HasPrefix(body.MIMEType, "multipart/") {
		for i, part := range body.Parts {
			partPath := path
			if partPath != "" {
				partPath += "."
			}
			partPath += fmt.Sprintf("%d", i+1)
			extractParts(c, seqset, part, partPath, content)
		}
		return
	}

	// Handle single part
	if body.MIMEType == "text/plain" || body.MIMEType == "text/html" {
		section := &imap.BodySectionName{}
		if path != "" {
			section.Specifier = imap.PartSpecifier(path)
		}

		messages := make(chan *imap.Message, 1)
		done := make(chan error, 1)
		go func() {
			done <- c.UidFetch(seqset, []imap.FetchItem{section.FetchItem()}, messages)
		}()

		for msg := range messages {
			if msg == nil {
				continue
			}
			if r := msg.GetBody(section); r != nil {
				b, err := io.ReadAll(r)
				if err == nil && len(b) > 0 {
					encoding := body.Encoding
					decoded := decodeTextContent(b, encoding)

					if body.MIMEType == "text/plain" {
						content.TextBody = decoded
					} else if body.MIMEType == "text/html" {
						content.HTMLBody = decoded
					}
					break
				}
			}
		}
		<-done
	} else {
		// Handle attachments
		section := &imap.BodySectionName{}
		if path != "" {
			section.Specifier = imap.PartSpecifier(path)
		}

		messages := make(chan *imap.Message, 1)
		done := make(chan error, 1)
		go func() {
			done <- c.UidFetch(seqset, []imap.FetchItem{section.FetchItem()}, messages)
		}()

		for msg := range messages {
			if msg == nil {
				continue
			}
			if r := msg.GetBody(section); r != nil {
				b, err := io.ReadAll(r)
				if err == nil && len(b) > 0 {
					filename := body.Description
					if filename == "" {
						filename = "attachment"
					}

					attachment := Attachment{
						Filename:    filename,
						ContentType: body.MIMEType,
						Size:        len(b),
						Data:        base64.StdEncoding.EncodeToString(b),
					}
					content.Attachments = append(content.Attachments, attachment)
					break
				}
			}
		}
		<-done
	}
}

func parseRawEmail(rawEmail string) (EmailContent, error) {
	// Parse the email using net/mail
	msg, err := mail.ReadMessage(strings.NewReader(rawEmail))
	if err != nil {
		// If parsing fails, try to extract content directly
		fmt.Printf("DEBUG: Failed to parse email with mail.ReadMessage: %v\n", err)
		return extractContentFromRaw(rawEmail), nil
	}

	content := EmailContent{
		Headers:     make(map[string]string),
		TextBody:    "",
		HTMLBody:    "",
		Attachments: []Attachment{},
	}

	// Extract headers
	for key, values := range msg.Header {
		content.Headers[key] = strings.Join(values, ", ")
	}

	// Parse the body
	mediaType, params, err := mime.ParseMediaType(msg.Header.Get("Content-Type"))
	if err != nil {
		// If we can't parse the content type, treat as plain text
		body, err := io.ReadAll(msg.Body)
		if err != nil {
			return EmailContent{}, fmt.Errorf("failed to read body: %v", err)
		}
		content.TextBody = string(body)
		return content, nil
	}

	if strings.HasPrefix(mediaType, "multipart/") {
		// Handle multipart messages
		boundary, ok := params["boundary"]
		if !ok {
			return EmailContent{}, fmt.Errorf("multipart message without boundary")
		}

		mr := multipart.NewReader(msg.Body, boundary)
		for {
			part, err := mr.NextPart()
			if err == io.EOF {
				break
			}
			if err != nil {
				continue
			}

			partData, err := io.ReadAll(part)
			if err != nil {
				continue
			}

			partMediaType, _, err := mime.ParseMediaType(part.Header.Get("Content-Type"))
			if err != nil {
				partMediaType = "text/plain"
			}

			switch partMediaType {
			case "text/plain":
				content.TextBody = string(partData)
			case "text/html":
				content.HTMLBody = string(partData)
			default:
				// Handle attachments
				filename := part.FileName()
				if filename == "" {
					filename = "attachment"
				}

				attachment := Attachment{
					Filename:    filename,
					ContentType: partMediaType,
					Size:        len(partData),
					Data:        base64.StdEncoding.EncodeToString(partData),
				}
				content.Attachments = append(content.Attachments, attachment)
			}
		}
	} else if mediaType == "text/plain" {
		body, err := io.ReadAll(msg.Body)
		if err != nil {
			return EmailContent{}, fmt.Errorf("failed to read text body: %v", err)
		}
		content.TextBody = string(body)
	} else if mediaType == "text/html" {
		body, err := io.ReadAll(msg.Body)
		if err != nil {
			return EmailContent{}, fmt.Errorf("failed to read html body: %v", err)
		}
		content.HTMLBody = string(body)
	}

	// If we still don't have content, try to extract from raw email
	if content.TextBody == "" && content.HTMLBody == "" {
		fmt.Printf("DEBUG: No content extracted from parsed email, trying raw extraction\n")
		rawContent := extractContentFromRaw(rawEmail)
		if rawContent.TextBody != "" || rawContent.HTMLBody != "" {
			return rawContent, nil
		}
	}

	return content, nil
}

// extractContentFromRaw tries to extract content from raw email when parsing fails
func extractContentFromRaw(rawEmail string) EmailContent {
	content := EmailContent{
		Headers:     make(map[string]string),
		TextBody:    "",
		HTMLBody:    "",
		Attachments: []Attachment{},
	}

	// Try to find content boundaries
	lines := strings.Split(rawEmail, "\n")
	var inBody bool
	var bodyLines []string

	for _, line := range lines {
		if strings.TrimSpace(line) == "" && !inBody {
			inBody = true
			continue
		}
		if inBody {
			bodyLines = append(bodyLines, line)
		}
	}

	if len(bodyLines) > 0 {
		bodyText := strings.Join(bodyLines, "\n")

		// Try to detect if it's HTML or plain text
		if strings.Contains(bodyText, "<html") || strings.Contains(bodyText, "<body") || strings.Contains(bodyText, "<div") {
			content.HTMLBody = bodyText
		} else {
			content.TextBody = bodyText
		}
	}

	return content
}

// min helper function for debug logging
func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}

func decodeTextContent(data []byte, encoding string) string {
	// Return raw content without any processing
	return string(data)
}

// decodeQuotedPrintable decodes quoted-printable encoded text
func decodeQuotedPrintable(input string) string {
	// Return raw content without any processing
	return input
}

// cleanTextContent removes common HTML entities and cleans up text
func cleanTextContent(text string) string {
	// Return raw content without any processing
	return text
}

// hexChar converts a hex character to its value
func hexChar(c byte) (byte, bool) {
	switch {
	case c >= '0' && c <= '9':
		return c - '0', true
	case c >= 'A' && c <= 'F':
		return c - 'A' + 10, true
	case c >= 'a' && c <= 'f':
		return c - 'a' + 10, true
	default:
		return 0, false
	}
}

func DownloadAttachment(ctx context.Context, uid uint32, filename string) (string, error) {
	// First, get the email content to find the attachment
	app := &App{}
	if app.Database == nil {
		app.Database, _ = NewDatabase()
	}

	account, err := app.Database.GetActiveAccount()
	if err != nil {
		return "", fmt.Errorf("failed to get active account: %v", err)
	}

	password, err := app.Database.GetPassword(account.ID)
	if err != nil {
		return "", fmt.Errorf("failed to get password: %v", err)
	}

	emailContent, err := FetchEmailContent(ctx, uid, account, password)
	if err != nil {
		return "", fmt.Errorf("failed to get email content: %v", err)
	}

	// Look for the attachment in the parsed content
	for _, attachment := range emailContent.Attachments {
		if attachment.Filename == filename {
			if attachment.Data == "" {
				return "", fmt.Errorf("attachment data not available")
			}
			return attachment.Data, nil
		}
	}

	return "", fmt.Errorf("attachment not found: %s", filename)
}

// markEmailAsReadOnServer marks an email as read on the IMAP server
func markEmailAsReadOnServer(ctx context.Context, emailID uint32, account *Account, password string) error {
	// Connect to server
	var c *client.Client
	var err error
	if account.UseSSL {
		c, err = client.DialTLS(fmt.Sprintf("%s:%d", account.EmailServer, account.Port), nil)
	} else {
		c, err = client.Dial(fmt.Sprintf("%s:%d", account.EmailServer, account.Port))
	}
	if err != nil {
		return err
	}
	defer c.Logout()

	// Login
	if err := c.Login(account.Username, password); err != nil {
		return err
	}

	// Select INBOX
	_, err = c.Select("INBOX", false)
	if err != nil {
		return err
	}

	// Create sequence set for the specific email
	seqset := new(imap.SeqSet)
	seqset.AddNum(emailID)

	// Mark the email as seen
	item := imap.FormatFlagsOp(imap.AddFlags, true)
	flags := []interface{}{imap.SeenFlag}
	err = c.UidStore(seqset, item, flags, nil)
	if err != nil {
		return fmt.Errorf("failed to mark email as read: %v", err)
	}

	return nil
}
