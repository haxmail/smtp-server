package backend

import (
	"smtpserver/src/session"

	"github.com/emersion/go-smtp"
)

type SmtpBackend struct{}

func (sb *SmtpBackend) NewSession(_ *smtp.Conn) (smtp.Session, error) {
	return &session.SmtpSession{}, nil
}
