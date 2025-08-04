package main

import (
	"context"
	"encoding/base64"
	"fmt"
	"io"
	"mime"
	"mime/multipart"
	"net/mail"
	"strings"

	"github.com/emersion/go-imap"
	"github.com/emersion/go-imap/client"
	"golang.org/x/text/encoding/charmap"
	"golang.org/x/text/transform"
)

type Email struct {
	ID      uint32 `json:"id"`
	From    string `json:"from"`
	Subject string `json:"subject"`
	Date    string `json:"date"`
	Snippet string `json:"snippet"`
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

func FetchEmails(ctx context.Context, page int, pageSize int) (EmailPage, error) {
	// Get active account from database
	app := &App{}
	if app.Database == nil {
		app.Database, _ = NewDatabase()
	}

	account, err := app.Database.GetActiveAccount()
	if err != nil {
		return EmailPage{}, fmt.Errorf("failed to get active account: %v", err)
	}

	password, err := app.Database.GetPassword(account.ID)
	if err != nil {
		return EmailPage{}, fmt.Errorf("failed to get password: %v", err)
	}

	// Connect to server
	var c *client.Client
	var err2 error
	if account.UseSSL {
		c, err2 = client.DialTLS(fmt.Sprintf("%s:%d", account.EmailServer, account.Port), nil)
	} else {
		c, err2 = client.Dial(fmt.Sprintf("%s:%d", account.EmailServer, account.Port))
	}
	if err2 != nil {
		return EmailPage{}, err2
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
	if total == 0 {
		return EmailPage{Emails: []Email{}, TotalCount: 0}, nil
	}

	// Calculate which messages to fetch (most recent first)
	var from, to uint32
	startIndex := uint32((page - 1) * pageSize)

	if startIndex >= total {
		return EmailPage{Emails: []Email{}, TotalCount: total}, nil
	}

	// Calculate the range (most recent messages first, so we count from the end)
	to = total - startIndex
	from = to - uint32(pageSize) + 1
	if from < 1 {
		from = 1
	}

	seqset := new(imap.SeqSet)
	seqset.AddRange(from, to)

	messages := make(chan *imap.Message, pageSize)
	done := make(chan error, 1)
	go func() {
		done <- c.Fetch(seqset, []imap.FetchItem{imap.FetchEnvelope, imap.FetchInternalDate, imap.FetchUid, imap.FetchFlags, imap.FetchBodyStructure}, messages)
	}()

	var emails []Email
	for msg := range messages {
		email := Email{
			ID:      msg.Uid,
			From:    msg.Envelope.From[0].MailboxName + "@" + msg.Envelope.From[0].HostName,
			Subject: msg.Envelope.Subject,
			Date:    msg.Envelope.Date.Format("2006-01-02"),
			Snippet: "",
		}
		fmt.Printf("Created email with ID: %d, Subject: %s\n", email.ID, email.Subject)
		emails = append([]Email{email}, emails...) // Prepend to reverse order (most recent first)
	}
	if err := <-done; err != nil {
		return EmailPage{}, err
	}
	return EmailPage{Emails: emails, TotalCount: total}, nil
}

func FetchFolders(ctx context.Context) ([]string, error) {
	// Get active account from database
	app := &App{}
	if app.Database == nil {
		app.Database, _ = NewDatabase()
	}

	account, err := app.Database.GetActiveAccount()
	if err != nil {
		return nil, fmt.Errorf("failed to get active account: %v", err)
	}

	password, err := app.Database.GetPassword(account.ID)
	if err != nil {
		return nil, fmt.Errorf("failed to get password: %v", err)
	}

	var c *client.Client
	var err2 error
	if account.UseSSL {
		c, err2 = client.DialTLS(fmt.Sprintf("%s:%d", account.EmailServer, account.Port), nil)
	} else {
		c, err2 = client.Dial(fmt.Sprintf("%s:%d", account.EmailServer, account.Port))
	}
	if err2 != nil {
		return nil, err2
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

func FetchEmailContent(ctx context.Context, uid uint32) (EmailContent, error) {
	// Get active account from database
	app := &App{}
	if app.Database == nil {
		app.Database, _ = NewDatabase()
	}

	account, err := app.Database.GetActiveAccount()
	if err != nil {
		return EmailContent{}, fmt.Errorf("failed to get active account: %v", err)
	}

	password, err := app.Database.GetPassword(account.ID)
	if err != nil {
		return EmailContent{}, fmt.Errorf("failed to get password: %v", err)
	}

	var c *client.Client
	var err2 error
	if account.UseSSL {
		c, err2 = client.DialTLS(fmt.Sprintf("%s:%d", account.EmailServer, account.Port), nil)
	} else {
		c, err2 = client.Dial(fmt.Sprintf("%s:%d", account.EmailServer, account.Port))
	}
	if err2 != nil {
		return EmailContent{}, err2
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

	section := &imap.BodySectionName{}
	messages := make(chan *imap.Message, 1)
	done := make(chan error, 1)
	go func() {
		done <- c.UidFetch(seqset, []imap.FetchItem{section.FetchItem()}, messages)
	}()

	var rawBody string
	for msg := range messages {
		if msg == nil {
			continue
		}
		if r := msg.GetBody(section); r != nil {
			b, err := io.ReadAll(r)
			if err != nil {
				continue
			}
			rawBody = string(b)
			if len(rawBody) > 0 {
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

	return content, nil
}

func parseRawEmail(rawEmail string) (EmailContent, error) {
	// Parse the email using net/mail
	msg, err := mail.ReadMessage(strings.NewReader(rawEmail))
	if err != nil {
		return EmailContent{}, fmt.Errorf("failed to parse email: %v", err)
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
				encoding := part.Header.Get("Content-Transfer-Encoding")
				content.TextBody = decodeTextContent(partData, encoding)
			case "text/html":
				encoding := part.Header.Get("Content-Transfer-Encoding")
				content.HTMLBody = decodeTextContent(partData, encoding)
			default:
				// Handle attachments
				filename := part.FileName()
				if filename == "" {
					filename = "attachment"
				}

				// Check if the data is already base64 encoded
				encoding := part.Header.Get("Content-Transfer-Encoding")
				var decodedData []byte

				if strings.ToLower(encoding) == "base64" {
					// Data is already base64 encoded, decode it
					decodedData, err = base64.StdEncoding.DecodeString(string(partData))
					if err != nil {
						decodedData = partData
					}
				} else {
					// Data is not base64 encoded, use as-is
					decodedData = partData
				}

				attachment := Attachment{
					Filename:    filename,
					ContentType: partMediaType,
					Size:        len(decodedData),
					Data:        base64.StdEncoding.EncodeToString(decodedData), // Store decoded data as base64
				}
				content.Attachments = append(content.Attachments, attachment)
			}
		}
	} else if mediaType == "text/plain" {
		body, err := io.ReadAll(msg.Body)
		if err != nil {
			return EmailContent{}, fmt.Errorf("failed to read text body: %v", err)
		}
		encoding := msg.Header.Get("Content-Transfer-Encoding")
		content.TextBody = decodeTextContent(body, encoding)
	} else if mediaType == "text/html" {
		body, err := io.ReadAll(msg.Body)
		if err != nil {
			return EmailContent{}, fmt.Errorf("failed to read html body: %v", err)
		}
		encoding := msg.Header.Get("Content-Transfer-Encoding")
		content.HTMLBody = decodeTextContent(body, encoding)
	}

	return content, nil
}

func decodeTextContent(data []byte, encoding string) string {
	// Handle different encodings
	switch strings.ToLower(encoding) {
	case "quoted-printable":
		// For now, just return as string - quoted-printable is mostly ASCII
		return string(data)
	case "base64":
		decoded, err := base64.StdEncoding.DecodeString(string(data))
		if err != nil {
			return string(data) // Return original if decoding fails
		}
		return string(decoded)
	case "7bit", "8bit":
		return string(data)
	default:
		// Try to detect and handle character encodings
		if strings.Contains(strings.ToLower(encoding), "iso-8859-1") {
			reader := transform.NewReader(strings.NewReader(string(data)), charmap.ISO8859_1.NewDecoder())
			decoded, err := io.ReadAll(reader)
			if err != nil {
				return string(data)
			}
			return string(decoded)
		}
		return string(data)
	}
}

func DownloadAttachment(ctx context.Context, uid uint32, filename string) (string, error) {
	// First, get the email content to find the attachment
	emailContent, err := FetchEmailContent(ctx, uid)
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
