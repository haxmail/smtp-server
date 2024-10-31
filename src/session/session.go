package session

import (
	"fmt"
	"io"
	"log"

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
	if b, err := io.ReadAll(r); err != nil {
		return err
	} else {
		log.Println("Received message: ", string(b))
		return nil
	}
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
