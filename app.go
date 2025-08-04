package main

import (
	"context"
	"encoding/base64"
	"fmt"
	"os"
	"path/filepath"
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

// GetEmails retrieves emails for the active account
func (a *App) GetEmails(page int, pageSize int) (EmailPage, error) {
	return FetchEmails(a.Ctx, page, pageSize)
}

// GetFolders retrieves folders for the active account
func (a *App) GetFolders() ([]string, error) {
	return FetchFolders(a.Ctx)
}

// GetEmailContent retrieves the content of a specific email
func (a *App) GetEmailContent(uid uint32) (EmailContent, error) {
	return FetchEmailContent(a.Ctx, uid)
}

// DownloadAttachment downloads a specific attachment from an email
func (a *App) DownloadAttachment(uid uint32, filename string) (string, error) {
	return DownloadAttachment(a.Ctx, uid, filename)
}

// SaveAttachment saves an attachment to the downloads folder
func (a *App) SaveAttachment(uid uint32, filename string) (string, error) {
	// Get the email content to find the attachment
	emailContent, err := FetchEmailContent(a.Ctx, uid)
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
