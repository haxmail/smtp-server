package main_test

import (
	"net/smtp"
	"testing"
)

func TestSendEmail(t *testing.T) {
	from := "sender@example.com"
	to := "recipient@example.com"
	subject := "Test Email"
	body := "This is a test email."

	// Connect to the SMTP server
	client, err := smtp.Dial("0.0.0.0:2525")
	if err != nil {
		t.Fatalf("Failed to connect to the SMTP server: %v", err)
	}
	defer client.Close()

	// Set the sender and recipient
	if err := client.Mail(from); err != nil {
		t.Fatalf("Failed to set the sender: %v", err)
	}
	if err := client.Rcpt(to); err != nil {
		t.Fatalf("Failed to set the recipient: %v", err)
	}

	// Start the data transfer
	w, err := client.Data()
	if err != nil {
		t.Fatalf("Failed to start the data transfer: %v", err)
	}
	defer w.Close()

	// Write the email headers and body
	email := "From: " + from + "\r\n" +
		"To: " + to + "\r\n" +
		"Subject: " + subject + "\r\n\r\n" +
		body
	if _, err := w.Write([]byte(email)); err != nil {
		t.Fatalf("Failed to write email data: %v", err)
	}

	// Send the email
	if err := client.Quit(); err != nil {
		t.Fatalf("Failed to send the email: %v", err)
	}
}
