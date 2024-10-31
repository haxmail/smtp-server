package db

import (
	"smtpserver/pkg/services/email"
)

var (
	EmailSvc email.Service = nil
)

func InitServices() {
	db := GetDB()
	emailRepo := email.NewPostgresRepo(db)
	EmailSvc = email.NewService(emailRepo)

}
