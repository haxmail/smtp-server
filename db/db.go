package db

import (
	"log"
	"smtpserver/config"

	"gorm.io/driver/postgres"
	"gorm.io/gorm"
)

var db *gorm.DB = nil

func GetDB() *gorm.DB {
	if db != nil {
		return db
	}

	db = Connect()
	return db
}

func Connect() *gorm.DB {

	db, err := gorm.Open(postgres.Open(config.DB_URI), &gorm.Config{})
	if err != nil {
		log.Println("Error connection to db", err)
	}
	return db
}
