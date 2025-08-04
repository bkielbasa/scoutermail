package main

import (
	"database/sql"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"time"

	_ "github.com/mattn/go-sqlite3"
	"github.com/zalando/go-keyring"
)

type Database struct {
	db *sql.DB
}

type Account struct {
	ID          int    `json:"id"`
	Name        string `json:"name"`
	EmailServer string `json:"emailServer"`
	Port        int    `json:"port"`
	UseSSL      bool   `json:"useSSL"`
	Username    string `json:"username"`
	IsActive    bool   `json:"isActive"`
}

type CachedEmail struct {
	ID        uint32    `json:"id"`
	From      string    `json:"from"`
	Subject   string    `json:"subject"`
	Date      string    `json:"date"`
	Snippet   string    `json:"snippet"`
	AccountID int       `json:"accountId"`
	CreatedAt time.Time `json:"createdAt"`
}

type CachedEmailContent struct {
	EmailID   uint32            `json:"emailId"`
	Headers   map[string]string `json:"headers"`
	TextBody  string            `json:"textBody"`
	HTMLBody  string            `json:"htmlBody"`
	CreatedAt time.Time         `json:"createdAt"`
}

func NewDatabase() (*Database, error) {
	// Get user's home directory
	homeDir, err := os.UserHomeDir()
	if err != nil {
		return nil, fmt.Errorf("failed to get home directory: %v", err)
	}

	// Create app directory
	appDir := filepath.Join(homeDir, ".scoutermail")
	if err := os.MkdirAll(appDir, 0755); err != nil {
		return nil, fmt.Errorf("failed to create app directory: %v", err)
	}

	// Open database
	dbPath := filepath.Join(appDir, "settings.db")
	db, err := sql.Open("sqlite3", dbPath)
	if err != nil {
		return nil, fmt.Errorf("failed to open database: %v", err)
	}

	// Create tables
	if err := createTables(db); err != nil {
		return nil, fmt.Errorf("failed to create tables: %v", err)
	}

	return &Database{db: db}, nil
}

func createTables(db *sql.DB) error {
	// Create accounts table
	accountsTable := `
	CREATE TABLE IF NOT EXISTS accounts (
		id INTEGER PRIMARY KEY AUTOINCREMENT,
		name TEXT NOT NULL,
		email_server TEXT NOT NULL,
		port INTEGER NOT NULL,
		use_ssl BOOLEAN NOT NULL,
		username TEXT NOT NULL,
		is_active BOOLEAN NOT NULL DEFAULT 0,
		created_at DATETIME DEFAULT CURRENT_TIMESTAMP
	);`

	// Create cached_emails table
	emailsTable := `
	CREATE TABLE IF NOT EXISTS cached_emails (
		id INTEGER PRIMARY KEY,
		from_address TEXT NOT NULL,
		subject TEXT NOT NULL,
		date TEXT NOT NULL,
		snippet TEXT,
		account_id INTEGER NOT NULL,
		read_status BOOLEAN DEFAULT 0,
		attachment_count INTEGER DEFAULT 0,
		created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
		FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
	);`

	// Create cached_email_content table
	emailContentTable := `
	CREATE TABLE IF NOT EXISTS cached_email_content (
		email_id INTEGER PRIMARY KEY,
		headers TEXT,
		text_body TEXT,
		html_body TEXT,
		created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
		FOREIGN KEY (email_id) REFERENCES cached_emails(id) ON DELETE CASCADE
	);`

	// Create cached_attachments table
	attachmentsTable := `
	CREATE TABLE IF NOT EXISTS cached_attachments (
		id INTEGER PRIMARY KEY AUTOINCREMENT,
		email_id INTEGER NOT NULL,
		filename TEXT NOT NULL,
		content_type TEXT NOT NULL,
		size INTEGER NOT NULL,
		data TEXT NOT NULL,
		created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
		FOREIGN KEY (email_id) REFERENCES cached_emails(id) ON DELETE CASCADE
	);`

	// Create user_preferences table
	preferencesTable := `
	CREATE TABLE IF NOT EXISTS user_preferences (
		key TEXT PRIMARY KEY,
		value TEXT NOT NULL,
		updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
	);`

	_, err := db.Exec(accountsTable)
	if err != nil {
		return fmt.Errorf("failed to create accounts table: %v", err)
	}

	_, err = db.Exec(emailsTable)
	if err != nil {
		return fmt.Errorf("failed to create cached_emails table: %v", err)
	}

	_, err = db.Exec(emailContentTable)
	if err != nil {
		return fmt.Errorf("failed to create cached_email_content table: %v", err)
	}

	_, err = db.Exec(attachmentsTable)
	if err != nil {
		return fmt.Errorf("failed to create cached_attachments table: %v", err)
	}

	_, err = db.Exec(preferencesTable)
	if err != nil {
		return fmt.Errorf("failed to create user_preferences table: %v", err)
	}

	// Run migrations to add missing columns
	if err := runMigrations(db); err != nil {
		return fmt.Errorf("failed to run migrations: %v", err)
	}

	return nil
}

// runMigrations adds missing columns to existing tables
func runMigrations(db *sql.DB) error {
	// Check if read_status column exists in cached_emails table
	var count int
	err := db.QueryRow("SELECT COUNT(*) FROM pragma_table_info('cached_emails') WHERE name = 'read_status'").Scan(&count)
	if err != nil {
		return fmt.Errorf("failed to check for read_status column: %v", err)
	}

	if count == 0 {
		fmt.Printf("DEBUG: Adding read_status column to cached_emails table\n")
		_, err = db.Exec("ALTER TABLE cached_emails ADD COLUMN read_status BOOLEAN DEFAULT 0")
		if err != nil {
			return fmt.Errorf("failed to add read_status column: %v", err)
		}
		fmt.Printf("DEBUG: read_status column added successfully\n")
	}

	// Check if attachment_count column exists in cached_emails table
	err = db.QueryRow("SELECT COUNT(*) FROM pragma_table_info('cached_emails') WHERE name = 'attachment_count'").Scan(&count)
	if err != nil {
		return fmt.Errorf("failed to check for attachment_count column: %v", err)
	}

	if count == 0 {
		fmt.Printf("DEBUG: Adding attachment_count column to cached_emails table\n")
		_, err = db.Exec("ALTER TABLE cached_emails ADD COLUMN attachment_count INTEGER DEFAULT 0")
		if err != nil {
			return fmt.Errorf("failed to add attachment_count column: %v", err)
		}
		fmt.Printf("DEBUG: attachment_count column added successfully\n")
	}

	return nil
}

func (d *Database) GetAccounts() ([]Account, error) {
	rows, err := d.db.Query(`
		SELECT id, name, email_server, port, use_ssl, username, is_active
		FROM accounts
		ORDER BY name
	`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var accounts []Account
	for rows.Next() {
		var account Account
		err := rows.Scan(&account.ID, &account.Name, &account.EmailServer, &account.Port, &account.UseSSL, &account.Username, &account.IsActive)
		if err != nil {
			return nil, err
		}
		accounts = append(accounts, account)
	}

	return accounts, nil
}

func (d *Database) GetActiveAccount() (*Account, error) {
	var account Account
	err := d.db.QueryRow(`
		SELECT id, name, email_server, port, use_ssl, username, is_active
		FROM accounts
		WHERE is_active = 1
		LIMIT 1
	`).Scan(&account.ID, &account.Name, &account.EmailServer, &account.Port, &account.UseSSL, &account.Username, &account.IsActive)
	if err != nil {
		if err == sql.ErrNoRows {
			return nil, fmt.Errorf("no active account found")
		}
		return nil, err
	}
	return &account, nil
}

// SetActiveAccount sets the specified account as active
func (d *Database) SetActiveAccount(accountID int) error {
	// Begin transaction
	tx, err := d.db.Begin()
	if err != nil {
		return err
	}
	defer tx.Rollback()

	// Set all accounts as inactive
	_, err = tx.Exec("UPDATE accounts SET is_active = 0")
	if err != nil {
		return fmt.Errorf("failed to deactivate all accounts: %v", err)
	}

	// Set the specified account as active
	_, err = tx.Exec("UPDATE accounts SET is_active = 1 WHERE id = ?", accountID)
	if err != nil {
		return fmt.Errorf("failed to activate account: %v", err)
	}

	return tx.Commit()
}

func (d *Database) GetAccountByID(id int) (*Account, error) {
	var account Account
	err := d.db.QueryRow(`
		SELECT id, name, email_server, port, use_ssl, username, is_active
		FROM accounts
		WHERE id = ?
	`, id).Scan(&account.ID, &account.Name, &account.EmailServer, &account.Port, &account.UseSSL, &account.Username, &account.IsActive)
	if err != nil {
		return nil, err
	}

	return &account, nil
}

func (d *Database) SaveAccount(account *Account, password string) error {
	// Begin transaction
	tx, err := d.db.Begin()
	if err != nil {
		return err
	}
	defer tx.Rollback()

	// If this is a new account (ID = 0), insert it
	if account.ID == 0 {
		result, err := tx.Exec(`
			INSERT INTO accounts (name, email_server, port, use_ssl, username, is_active)
			VALUES (?, ?, ?, ?, ?, ?)
		`, account.Name, account.EmailServer, account.Port, account.UseSSL, account.Username, account.IsActive)
		if err != nil {
			return err
		}

		// Get the inserted ID
		id, err := result.LastInsertId()
		if err != nil {
			return err
		}
		account.ID = int(id)
	} else {
		// Update existing account
		_, err = tx.Exec(`
			UPDATE accounts 
			SET name = ?, email_server = ?, port = ?, use_ssl = ?, username = ?, is_active = ?
			WHERE id = ?
		`, account.Name, account.EmailServer, account.Port, account.UseSSL, account.Username, account.IsActive, account.ID)
		if err != nil {
			return err
		}
	}

	// If this account is being set as active, deactivate others
	if account.IsActive {
		_, err = tx.Exec("UPDATE accounts SET is_active = 0 WHERE id != ?", account.ID)
		if err != nil {
			return err
		}
	}

	// Save password to keyring
	if password != "" {
		keyName := fmt.Sprintf("account_%d", account.ID)
		err = keyring.Set("scoutermail", keyName, password)
		if err != nil {
			return fmt.Errorf("failed to save password to keyring: %v", err)
		}
	}

	return tx.Commit()
}

func (d *Database) DeleteAccount(id int) error {
	// Begin transaction
	tx, err := d.db.Begin()
	if err != nil {
		return err
	}
	defer tx.Rollback()

	// Delete from database
	_, err = tx.Exec("DELETE FROM accounts WHERE id = ?", id)
	if err != nil {
		return err
	}

	// Delete password from keyring
	keyName := fmt.Sprintf("account_%d", id)
	keyring.Delete("scoutermail", keyName)

	return tx.Commit()
}

func (d *Database) GetPassword(accountID int) (string, error) {
	keyName := fmt.Sprintf("account_%d", accountID)
	password, err := keyring.Get("scoutermail", keyName)
	if err != nil {
		return "", err
	}
	return password, nil
}

// CacheEmails stores emails in the database for faster access
func (d *Database) CacheEmails(accountID int, emails []Email) error {
	// Start a transaction
	tx, err := d.db.Begin()
	if err != nil {
		return fmt.Errorf("failed to begin transaction: %v", err)
	}
	defer tx.Rollback()

	// Delete existing emails for this account
	_, err = tx.Exec("DELETE FROM cached_emails WHERE account_id = ?", accountID)
	if err != nil {
		return fmt.Errorf("failed to delete existing emails: %v", err)
	}

	// Insert new emails
	stmt, err := tx.Prepare(`
		INSERT INTO cached_emails (id, from_address, subject, date, snippet, account_id, read_status, attachment_count)
		VALUES (?, ?, ?, ?, ?, ?, ?, ?)
	`)
	if err != nil {
		return fmt.Errorf("failed to prepare insert statement: %v", err)
	}
	defer stmt.Close()

	for _, email := range emails {
		_, err = stmt.Exec(email.ID, email.From, email.Subject, email.Date, email.Snippet, accountID, email.Read, email.AttachmentCount)
		if err != nil {
			return fmt.Errorf("failed to insert email: %v", err)
		}
	}

	return tx.Commit()
}

// GetCachedEmails retrieves cached emails for an account
func (d *Database) GetCachedEmails(accountID int) ([]Email, error) {
	rows, err := d.db.Query(`
		SELECT id, from_address, subject, date, snippet, read_status, attachment_count
		FROM cached_emails 
		WHERE account_id = ?
		ORDER BY id DESC
	`, accountID)
	if err != nil {
		return nil, fmt.Errorf("failed to query cached emails: %v", err)
	}
	defer rows.Close()

	var emails []Email
	for rows.Next() {
		var email Email
		err := rows.Scan(&email.ID, &email.From, &email.Subject, &email.Date, &email.Snippet, &email.Read, &email.AttachmentCount)
		if err != nil {
			return nil, fmt.Errorf("failed to scan email: %v", err)
		}
		emails = append(emails, email)
	}

	return emails, nil
}

// MarkEmailAsRead marks an email as read
func (d *Database) MarkEmailAsRead(emailID uint32) error {
	_, err := d.db.Exec("UPDATE cached_emails SET read_status = 1 WHERE id = ?", emailID)
	if err != nil {
		return fmt.Errorf("failed to mark email as read: %v", err)
	}
	return nil
}

// CacheEmailContent stores email content in the database
func (d *Database) CacheEmailContent(emailID uint32, content EmailContent) error {
	headersJSON, err := json.Marshal(content.Headers)
	if err != nil {
		return fmt.Errorf("failed to marshal headers: %v", err)
	}

	// Store the main content
	_, err = d.db.Exec(`
		INSERT OR REPLACE INTO cached_email_content (email_id, headers, text_body, html_body)
		VALUES (?, ?, ?, ?)
	`, emailID, string(headersJSON), content.TextBody, content.HTMLBody)
	if err != nil {
		return fmt.Errorf("failed to cache email content: %v", err)
	}

	// Store attachments if any
	if len(content.Attachments) > 0 {
		// First, clear existing attachments for this email
		_, err = d.db.Exec("DELETE FROM cached_attachments WHERE email_id = ?", emailID)
		if err != nil {
			return fmt.Errorf("failed to clear existing attachments: %v", err)
		}

		// Insert new attachments
		for _, attachment := range content.Attachments {
			_, err = d.db.Exec(`
				INSERT INTO cached_attachments (email_id, filename, content_type, size, data)
				VALUES (?, ?, ?, ?, ?)
			`, emailID, attachment.Filename, attachment.ContentType, attachment.Size, attachment.Data)
			if err != nil {
				return fmt.Errorf("failed to cache attachment %s: %v", attachment.Filename, err)
			}
		}
	}

	return nil
}

// GetCachedEmailContent retrieves cached email content
func (d *Database) GetCachedEmailContent(emailID uint32) (EmailContent, error) {
	var content EmailContent
	var headersJSON string

	err := d.db.QueryRow(`
		SELECT headers, text_body, html_body
		FROM cached_email_content
		WHERE email_id = ?
	`, emailID).Scan(&headersJSON, &content.TextBody, &content.HTMLBody)
	if err != nil {
		return EmailContent{}, fmt.Errorf("failed to get cached email content: %v", err)
	}

	// Parse headers JSON
	err = json.Unmarshal([]byte(headersJSON), &content.Headers)
	if err != nil {
		return EmailContent{}, fmt.Errorf("failed to unmarshal headers: %v", err)
	}

	// Get attachments for this email
	rows, err := d.db.Query(`
		SELECT filename, content_type, size, data
		FROM cached_attachments
		WHERE email_id = ?
	`, emailID)
	if err != nil {
		// Don't fail if attachments table doesn't exist or query fails
		fmt.Printf("Warning: failed to get cached attachments: %v\n", err)
		return content, nil
	}
	defer rows.Close()

	for rows.Next() {
		var attachment Attachment
		err := rows.Scan(&attachment.Filename, &attachment.ContentType, &attachment.Size, &attachment.Data)
		if err != nil {
			fmt.Printf("Warning: failed to scan attachment: %v\n", err)
			continue
		}
		content.Attachments = append(content.Attachments, attachment)
	}

	return content, nil
}

// ClearEmailCache clears all cached emails for the given account
func (d *Database) ClearEmailCache(accountID int) error {
	fmt.Printf("DEBUG: Clearing email cache for account %d\n", accountID)
	_, err := d.db.Exec("DELETE FROM cached_emails WHERE account_id = ?", accountID)
	if err != nil {
		fmt.Printf("DEBUG: Failed to clear email cache: %v\n", err)
		return fmt.Errorf("failed to clear email cache: %v", err)
	}
	fmt.Printf("DEBUG: Email cache cleared successfully\n")
	return nil
}

// ClearEmailContentCache clears all cached email content for the given account
func (d *Database) ClearEmailContentCache(accountID int) error {
	fmt.Printf("DEBUG: Clearing email content cache for account %d\n", accountID)

	// Get all email IDs for this account
	rows, err := d.db.Query("SELECT id FROM cached_emails WHERE account_id = ?", accountID)
	if err != nil {
		fmt.Printf("DEBUG: Failed to get email IDs: %v\n", err)
		return fmt.Errorf("failed to get email IDs: %v", err)
	}
	defer rows.Close()

	var emailIDs []uint32
	for rows.Next() {
		var emailID uint32
		if err := rows.Scan(&emailID); err != nil {
			fmt.Printf("DEBUG: Failed to scan email ID: %v\n", err)
			return fmt.Errorf("failed to scan email ID: %v", err)
		}
		emailIDs = append(emailIDs, emailID)
	}
	fmt.Printf("DEBUG: Found %d email IDs to clear content for\n", len(emailIDs))

	// Delete cached content for each email
	for _, emailID := range emailIDs {
		// Delete email content
		_, err := d.db.Exec("DELETE FROM cached_email_content WHERE email_id = ?", emailID)
		if err != nil {
			fmt.Printf("DEBUG: Failed to delete cached content for email %d: %v\n", emailID, err)
			return fmt.Errorf("failed to delete cached content for email %d: %v", emailID, err)
		}

		// Delete attachments
		_, err = d.db.Exec("DELETE FROM cached_attachments WHERE email_id = ?", emailID)
		if err != nil {
			// Don't fail if attachments table doesn't exist
			fmt.Printf("DEBUG: Warning: failed to delete cached attachments for email %d: %v\n", emailID, err)
		}
	}

	fmt.Printf("DEBUG: Email content cache cleared successfully\n")
	return nil
}

func (d *Database) Close() error {
	return d.db.Close()
}

// GetUserPreference retrieves a user preference by key
func (d *Database) GetUserPreference(key string) (string, error) {
	var value string
	err := d.db.QueryRow("SELECT value FROM user_preferences WHERE key = ?", key).Scan(&value)
	if err != nil {
		if err == sql.ErrNoRows {
			return "", nil // Return empty string if preference doesn't exist
		}
		return "", fmt.Errorf("failed to get user preference %s: %v", key, err)
	}
	return value, nil
}

// SetUserPreference sets a user preference
func (d *Database) SetUserPreference(key, value string) error {
	_, err := d.db.Exec(`
		INSERT OR REPLACE INTO user_preferences (key, value, updated_at) 
		VALUES (?, ?, CURRENT_TIMESTAMP)
	`, key, value)
	if err != nil {
		return fmt.Errorf("failed to set user preference %s: %v", key, err)
	}
	return nil
}

// GetUserPreferences retrieves all user preferences
func (d *Database) GetUserPreferences() (map[string]string, error) {
	rows, err := d.db.Query("SELECT key, value FROM user_preferences")
	if err != nil {
		return nil, fmt.Errorf("failed to query user preferences: %v", err)
	}
	defer rows.Close()

	preferences := make(map[string]string)
	for rows.Next() {
		var key, value string
		err := rows.Scan(&key, &value)
		if err != nil {
			return nil, fmt.Errorf("failed to scan user preference: %v", err)
		}
		preferences[key] = value
	}

	return preferences, nil
}

// DeleteUserPreference deletes a user preference
func (d *Database) DeleteUserPreference(key string) error {
	_, err := d.db.Exec("DELETE FROM user_preferences WHERE key = ?", key)
	if err != nil {
		return fmt.Errorf("failed to delete user preference %s: %v", key, err)
	}
	return nil
}
