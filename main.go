package main

import (
	"io/ioutil"
	"fmt"
)

func main() {
	bytes, err := ioutil.ReadFile("encoded.dab")
	if err != nil {
		fmt.Println(err)
		return
	}
	fmt.Print("[")
	for _, b := range bytes {
		fmt.Print(b)
		fmt.Print(", ")
	}
	fmt.Print("]")
}