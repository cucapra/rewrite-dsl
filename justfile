tests :=  "tests/printer/*.rw tests/checker/*.rw"

test *args:
	turnt {{tests}} {{args}}
