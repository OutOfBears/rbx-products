arg1 := $(word 2, $(MAKECMDGOALS))

.PHONY: all none release delete-release $(arg1)
.SILENT: none release delete-release $(arg1)

none:
	echo Please specify a target: build

release:
	git tag -a $(arg1) -m "Release $(arg1)"
	git push origin $(arg1)

delete-release:
	git tag -d $(arg1)
	git push --delete origin $(arg1)

all: none