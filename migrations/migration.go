package migrations

import (
	"smtpserver/db"
	"smtpserver/pkg/models"
)

func Migrate() {
	database := db.GetDB()
	database.AutoMigrate(&models.Email{})
}
