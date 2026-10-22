TESTS :=  tests/printer/*.rw

.PHONY: test
test:
	turnt $(TESTS)
