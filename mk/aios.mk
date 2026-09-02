# Redox AIOS — Makefile targets (incluir no Makefile upstream após overlay)

aios-minimal:
	$(MAKE) CONFIG_NAME=aios-minimal

aios:
	$(MAKE) CONFIG_NAME=aios

.PHONY: aios-minimal aios
