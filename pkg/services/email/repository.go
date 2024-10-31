package email

import "smtpserver/pkg/models"

type Repository interface {
	CreateEmail(email models.Email) (*models.Email, error)
}
