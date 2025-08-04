package main

import (
	"context"
	"encoding/base64"
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"strings"
)

// App struct
type App struct {
	Ctx      context.Context
	Database *Database
}

// NewApp creates a new App application struct
func NewApp() *App {
	db, err := NewDatabase()
	if err != nil {
		fmt.Printf("Failed to initialize database: %v\n", err)
		return &App{}
	}
	return &App{Database: db}
}

// startup is called when the app starts. The context is saved
// so we can call the runtime methods
func (a *App) startup(ctx context.Context) {
	a.Ctx = ctx
}

// Greet returns a greeting for the given name
func (a *App) Greet(name string) string {
	return fmt.Sprintf("Hello %s, It's show time!", name)
}

// GetEmails retrieves emails with caching support
func (a *App) GetEmails(page int, pageSize int) (EmailPage, error) {
	// Get active account
	account, err := a.Database.GetActiveAccount()
	if err != nil {
		return EmailPage{}, fmt.Errorf("failed to get active account: %v", err)
	}

	// Try to get cached emails first
	cachedEmails, err := a.Database.GetCachedEmails(account.ID)
	if err == nil && len(cachedEmails) > 0 {
		// Return cached emails with pagination
		start := (page - 1) * pageSize
		end := start + pageSize
		if start >= len(cachedEmails) {
			return EmailPage{Emails: []Email{}, TotalCount: uint32(len(cachedEmails))}, nil
		}
		if end > len(cachedEmails) {
			end = len(cachedEmails)
		}

		return EmailPage{
			Emails:     cachedEmails[start:end],
			TotalCount: uint32(len(cachedEmails)),
		}, nil
	}

	// If no cache, fetch from server
	password, err := a.Database.GetPassword(account.ID)
	if err != nil {
		return EmailPage{}, fmt.Errorf("failed to get password: %v", err)
	}

	return FetchEmails(a.Ctx, page, pageSize, account, password)
}

// RefreshEmails fetches fresh emails from the server and updates the cache
func (a *App) RefreshEmails() error {
	fmt.Printf("DEBUG: Starting email refresh...\n")

	// Get active account
	account, err := a.Database.GetActiveAccount()
	if err != nil {
		fmt.Printf("DEBUG: Failed to get active account: %v\n", err)
		return fmt.Errorf("failed to get active account: %v", err)
	}
	fmt.Printf("DEBUG: Using account: %s (%s)\n", account.Name, account.Username)

	// Get password
	password, err := a.Database.GetPassword(account.ID)
	if err != nil {
		fmt.Printf("DEBUG: Failed to get password: %v\n", err)
		return fmt.Errorf("failed to get password: %v", err)
	}
	fmt.Printf("DEBUG: Password retrieved successfully\n")

	// Clear existing cache (both emails and content)
	fmt.Printf("DEBUG: Clearing email cache...\n")
	err = a.Database.ClearEmailCache(account.ID)
	if err != nil {
		fmt.Printf("DEBUG: Failed to clear email cache: %v\n", err)
		return fmt.Errorf("failed to clear email cache: %v", err)
	}
	fmt.Printf("DEBUG: Email cache cleared\n")

	// Also clear email content cache to force fresh content
	fmt.Printf("DEBUG: Clearing email content cache...\n")
	err = a.Database.ClearEmailContentCache(account.ID)
	if err != nil {
		fmt.Printf("DEBUG: Failed to clear email content cache: %v\n", err)
		return fmt.Errorf("failed to clear email content cache: %v", err)
	}
	fmt.Printf("DEBUG: Email content cache cleared\n")

	// Fetch fresh emails (first 100 emails)
	fmt.Printf("DEBUG: Fetching emails from server...\n")
	emailPage, err := FetchEmails(a.Ctx, 1, 100, account, password)
	if err != nil {
		fmt.Printf("DEBUG: Failed to fetch emails: %v\n", err)
		return fmt.Errorf("failed to fetch emails: %v", err)
	}
	fmt.Printf("DEBUG: Fetched %d emails from server\n", len(emailPage.Emails))

	// Cache the emails
	fmt.Printf("DEBUG: Caching emails...\n")
	err = a.Database.CacheEmails(account.ID, emailPage.Emails)
	if err != nil {
		fmt.Printf("DEBUG: Failed to cache emails: %v\n", err)
		return fmt.Errorf("failed to cache emails: %v", err)
	}
	fmt.Printf("DEBUG: Emails cached successfully\n")

	fmt.Printf("Successfully refreshed %d emails for account %s\n", len(emailPage.Emails), account.Name)
	return nil
}

// ClearEmailContentCache clears the cached email content for the active account
func (a *App) ClearEmailContentCache() error {
	// Get active account
	account, err := a.Database.GetActiveAccount()
	if err != nil {
		return fmt.Errorf("failed to get active account: %v", err)
	}

	// Clear email content cache
	err = a.Database.ClearEmailContentCache(account.ID)
	if err != nil {
		return fmt.Errorf("failed to clear email content cache: %v", err)
	}

	return nil
}

// GetFolders retrieves folders for the active account
func (a *App) GetFolders() ([]string, error) {
	// Get active account
	account, err := a.Database.GetActiveAccount()
	if err != nil {
		return nil, fmt.Errorf("failed to get active account: %v", err)
	}

	// Get password
	password, err := a.Database.GetPassword(account.ID)
	if err != nil {
		return nil, fmt.Errorf("failed to get password: %v", err)
	}

	return FetchFolders(a.Ctx, account, password)
}

// GetEmailContent retrieves email content with caching support
func (a *App) GetEmailContent(uid uint32) (EmailContent, error) {
	// Try to get cached content first
	cachedContent, err := a.Database.GetCachedEmailContent(uid)
	if err == nil && (cachedContent.TextBody != "" || cachedContent.HTMLBody != "") {
		// Only return cached content if it has actual content
		return cachedContent, nil
	}

	// If not cached or empty, fetch from server
	// Get active account
	account, err := a.Database.GetActiveAccount()
	if err != nil {
		return EmailContent{}, fmt.Errorf("failed to get active account: %v", err)
	}

	// Get password
	password, err := a.Database.GetPassword(account.ID)
	if err != nil {
		return EmailContent{}, fmt.Errorf("failed to get password: %v", err)
	}

	content, err := FetchEmailContent(a.Ctx, uid, account, password)
	if err != nil {
		return EmailContent{}, err
	}

	// Cache the content (including attachments for better functionality)
	err = a.Database.CacheEmailContent(uid, content)
	if err != nil {
		// Log error but don't fail the request
		fmt.Printf("Failed to cache email content: %v\n", err)
	}

	return content, nil
}

// DownloadAttachment downloads a specific attachment from an email
func (a *App) DownloadAttachment(uid uint32, filename string) (string, error) {
	return DownloadAttachment(a.Ctx, uid, filename)
}

// SaveAttachment saves an attachment to the downloads folder
func (a *App) SaveAttachment(uid uint32, filename string) (string, error) {
	// Get active account
	account, err := a.Database.GetActiveAccount()
	if err != nil {
		return "", fmt.Errorf("failed to get active account: %v", err)
	}

	// Get password
	password, err := a.Database.GetPassword(account.ID)
	if err != nil {
		return "", fmt.Errorf("failed to get password: %v", err)
	}

	// Get the email content to find the attachment
	emailContent, err := FetchEmailContent(a.Ctx, uid, account, password)
	if err != nil {
		return "", fmt.Errorf("failed to get email content: %v", err)
	}

	// Look for the attachment in the parsed content
	for _, attachment := range emailContent.Attachments {
		if attachment.Filename == filename {
			if attachment.Data == "" {
				return "", fmt.Errorf("attachment data not available")
			}

			// Decode base64 data
			data, err := base64.StdEncoding.DecodeString(attachment.Data)
			if err != nil {
				return "", fmt.Errorf("failed to decode attachment data: %v", err)
			}

			// Get downloads folder
			homeDir, err := os.UserHomeDir()
			if err != nil {
				return "", fmt.Errorf("failed to get home directory: %v", err)
			}
			downloadsDir := filepath.Join(homeDir, "Downloads")

			// Create file path
			filePath := filepath.Join(downloadsDir, filename)

			// Write file
			err = os.WriteFile(filePath, data, 0644)
			if err != nil {
				return "", fmt.Errorf("failed to write file: %v", err)
			}

			return filePath, nil
		}
	}

	return "", fmt.Errorf("attachment not found: %s", filename)
}

// GetAccounts retrieves all email accounts
func (a *App) GetAccounts() ([]Account, error) {
	if a.Database == nil {
		return nil, fmt.Errorf("database not initialized")
	}
	return a.Database.GetAccounts()
}

// GetActiveAccount retrieves the currently active account
func (a *App) GetActiveAccount() (*Account, error) {
	if a.Database == nil {
		return nil, fmt.Errorf("database not initialized")
	}
	return a.Database.GetActiveAccount()
}

// SetActiveAccount sets the specified account as active
func (a *App) SetActiveAccount(accountID int) error {
	if a.Database == nil {
		return fmt.Errorf("database not initialized")
	}
	return a.Database.SetActiveAccount(accountID)
}

// SaveAccount saves an email account
func (a *App) SaveAccount(account *Account, password string) error {
	if a.Database == nil {
		return fmt.Errorf("database not initialized")
	}
	return a.Database.SaveAccount(account, password)
}

// DeleteAccount deletes an email account
func (a *App) DeleteAccount(id int) error {
	if a.Database == nil {
		return fmt.Errorf("database not initialized")
	}
	return a.Database.DeleteAccount(id)
}

// GetPassword retrieves the password for an account
func (a *App) GetPassword(accountID int) (string, error) {
	if a.Database == nil {
		return "", fmt.Errorf("database not initialized")
	}
	return a.Database.GetPassword(accountID)
}

// GetUserPreference retrieves a user preference
func (a *App) GetUserPreference(key string) (string, error) {
	if a.Database == nil {
		return "", fmt.Errorf("database not initialized")
	}
	return a.Database.GetUserPreference(key)
}

// SetUserPreference sets a user preference
func (a *App) SetUserPreference(key, value string) error {
	if a.Database == nil {
		return fmt.Errorf("database not initialized")
	}
	return a.Database.SetUserPreference(key, value)
}

// GetUserPreferences retrieves all user preferences
func (a *App) GetUserPreferences() (map[string]string, error) {
	if a.Database == nil {
		return nil, fmt.Errorf("database not initialized")
	}
	return a.Database.GetUserPreferences()
}

// DeleteUserPreference deletes a user preference
func (a *App) DeleteUserPreference(key string) error {
	if a.Database == nil {
		return fmt.Errorf("database not initialized")
	}
	return a.Database.DeleteUserPreference(key)
}

// MarkEmailAsRead marks an email as read
func (a *App) MarkEmailAsRead(emailID uint32) error {
	// Mark as read in local database
	err := a.Database.MarkEmailAsRead(emailID)
	if err != nil {
		return fmt.Errorf("failed to mark email as read in database: %v", err)
	}

	// Get active account
	account, err := a.Database.GetActiveAccount()
	if err != nil {
		return fmt.Errorf("failed to get active account: %v", err)
	}

	// Get password
	password, err := a.Database.GetPassword(account.ID)
	if err != nil {
		return fmt.Errorf("failed to get password: %v", err)
	}

	// Mark as read on IMAP server
	err = markEmailAsReadOnServer(a.Ctx, emailID, account, password)
	if err != nil {
		// Log error but don't fail the operation
		fmt.Printf("Warning: failed to mark email as read on server: %v\n", err)
	}

	return nil
}

// ForceRefreshEmailContent forces a refresh of a specific email's content
func (a *App) ForceRefreshEmailContent(uid uint32) error {
	// Get active account
	account, err := a.Database.GetActiveAccount()
	if err != nil {
		return fmt.Errorf("failed to get active account: %v", err)
	}

	// Get password
	password, err := a.Database.GetPassword(account.ID)
	if err != nil {
		return fmt.Errorf("failed to get password: %v", err)
	}

	// Clear cached content for this specific email
	err = a.Database.ClearEmailContentCache(account.ID)
	if err != nil {
		return fmt.Errorf("failed to clear email content cache: %v", err)
	}

	// Fetch fresh content
	content, err := FetchEmailContent(a.Ctx, uid, account, password)
	if err != nil {
		return fmt.Errorf("failed to fetch email content: %v", err)
	}

	// Cache the fresh content
	err = a.Database.CacheEmailContent(uid, content)
	if err != nil {
		return fmt.Errorf("failed to cache email content: %v", err)
	}

	return nil
}

// GetEmailSnippet generates a proper snippet for a specific email
func (a *App) GetEmailSnippet(emailID uint32) (string, error) {
	// Get active account
	account, err := a.Database.GetActiveAccount()
	if err != nil {
		return "", fmt.Errorf("failed to get active account: %v", err)
	}

	// Get password
	password, err := a.Database.GetPassword(account.ID)
	if err != nil {
		return "", fmt.Errorf("failed to get password: %v", err)
	}

	// Fetch email content to generate snippet
	content, err := FetchEmailContent(a.Ctx, emailID, account, password)
	if err != nil {
		return "", fmt.Errorf("failed to fetch email content: %v", err)
	}

	// Generate snippet from content
	var snippet string
	if content.TextBody != "" {
		snippet = content.TextBody
	} else if content.HTMLBody != "" {
		// Simple HTML tag removal for snippet
		snippet = content.HTMLBody
		snippet = strings.ReplaceAll(snippet, "<br>", " ")
		snippet = strings.ReplaceAll(snippet, "<br/>", " ")
		snippet = strings.ReplaceAll(snippet, "<br />", " ")
		snippet = strings.ReplaceAll(snippet, "</p>", " ")
		snippet = strings.ReplaceAll(snippet, "</div>", " ")

		// Remove HTML tags
		re := regexp.MustCompile(`<[^>]*>`)
		snippet = re.ReplaceAllString(snippet, "")
	}

	// Clean and limit snippet
	if snippet != "" {
		// Remove extra whitespace
		snippet = strings.Join(strings.Fields(snippet), " ")

		// Limit to 150 characters
		if len(snippet) > 150 {
			snippet = snippet[:150] + "..."
		}
	} else {
		snippet = "No preview available"
	}

	return snippet, nil
}
