package main

import (
	"database/sql"
	"fmt"
	"os"
	"path/filepath"

	_ "github.com/mattn/go-sqlite3"
	"github.com/zalando/go-keyring"
)

type Account struct {
	ID          int    `json:"id"`
	Name        string `json:"name"`
	EmailServer string `json:"emailServer"`
	Port        int    `json:"port"`
	UseSSL      bool   `json:"useSSL"`
	Username    string `json:"username"`
	IsActive    bool   `json:"isActive"`
}

type Settings struct {
	EmailServer string `json:"emailServer"`
	Port        int    `json:"port"`
	UseSSL      bool   `json:"useSSL"`
	Username    string `json:"username"`
}

type Database struct {
	db *sql.DB
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
	// Accounts table
	_, err := db.Exec(`
		CREATE TABLE IF NOT EXISTS accounts (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			name TEXT NOT NULL,
			email_server TEXT NOT NULL,
			port INTEGER NOT NULL,
			use_ssl BOOLEAN NOT NULL,
			username TEXT NOT NULL,
			is_active BOOLEAN DEFAULT 0,
			created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
			updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
		)
	`)
	if err != nil {
		return err
	}

	// Insert default account if table is empty
	var count int
	err = db.QueryRow("SELECT COUNT(*) FROM accounts").Scan(&count)
	if err != nil {
		return err
	}

	if count == 0 {
		_, err = db.Exec(`
			INSERT INTO accounts (name, email_server, port, use_ssl, username, is_active)
			VALUES (?, ?, ?, ?, ?, ?)
		`, "Default Account", "imap.gmail.com", 993, true, "", 1)
		if err != nil {
			return err
		}
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
		return nil, err
	}

	return &account, nil
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

func (d *Database) Close() error {
	return d.db.Close()
}
