package session

import (
	"fmt"
	"io"
	"log"
	"smtpserver/db"
	"smtpserver/pkg/models"

	"github.com/DusanKasan/parsemail"
	"github.com/emersion/go-smtp"
)

type SmtpSession struct {
	From string
	To   []string
}

func (s *SmtpSession) Mail(from string, opts *smtp.MailOptions) error {
	log.Println("Mail from: ", from)
	s.From = from
	return nil
}

func (s *SmtpSession) Rcpt(to string, opts *smtp.RcptOptions) error {
	log.Println("Rcpt To: ", to)
	s.To = append(s.To, to)
	return nil
}

func (s *SmtpSession) Data(r io.Reader) error {
	/*b, err := io.ReadAll(r)
	if err != nil {
		return err
	}
	log.Println("Received message: ", string(b))*/
	data, err := parsemail.Parse(r)
	if err != nil {
		return err
	}
	log.Println("Parsed email data: ", data)
	body := data.HTMLBody
	if data.HTMLBody == "" {
		body = data.TextBody
	}
	email := models.Email{
		From:    s.From,
		To:      s.To[0],
		Subject: data.Subject,
		Body:    body,
	}
	newEmail, err := db.EmailSvc.CreateEmail(email)
	if err != nil {
		log.Println(err)
	}
	log.Println(newEmail)
	return nil

}

func (s *SmtpSession) AuthPlain(username, password string) error {
	if username != "testuser" || password != "testpass" {
		return fmt.Errorf("invalid username of password")
	}
	return nil
}

func (s *SmtpSession) Reset() {
	s.From = ""
	s.To = make([]string, 0)
}

func (s *SmtpSession) Logout() error {
	return nil
}
