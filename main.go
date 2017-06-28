package main

import (
	"io/ioutil"
	"log"
)

func main() {
	bytes, err := ioutil.ReadFile("encoded.dab")
	if err != nil {
		log.Fatalln(err)
		return
	}
	log.Println(bytes)
}