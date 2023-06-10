package controllers

import (
	"fmt"
	"smtpserver/src/models"

	"gorm.io/gorm"
)

func processMail(from string, to string, data string, db *gorm.DB) {
	mail := models.Mail{From: from, To: to, Data: data}
	db.Create(&mail)
	fmt.Printf("Mail from %s to %s saved\n", from, to)
}
