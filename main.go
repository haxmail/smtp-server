package main

import (
	"log"
	"smtpserver/src/backend"
	"time"

	"github.com/emersion/go-smtp"
)

func main() {
	s := smtp.NewServer(&backend.SmtpBackend{})
	s.Addr = ":2525"
	s.Domain = "indiedev.blog"
	s.WriteTimeout = 10 * time.Second
	s.ReadTimeout = 10 * time.Second
	s.MaxMessageBytes = 1024 * 1024
	s.MaxRecipients = 50
	s.AllowInsecureAuth = true

	log.Println("Starting server at: ", s.Addr)
	if err := s.ListenAndServe(); err != nil {
		log.Fatal(err)
	}
}
