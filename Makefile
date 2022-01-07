FUNCTIONS := handler
STACK_NAME := rust-sqs
ARCH := aarch64-unknown-linux-gnu

build:
	rm -rf ./build
	rm -rf ./target
	cross build --release --target $(ARCH)
	mkdir -p ./build
	${MAKE} ${MAKEOPTS} $(foreach function,${FUNCTIONS}, build-${function})

build-%:
	mkdir -p ./build/$*
	cp -v ./target/$(ARCH)/release/$* ./build/$*/bootstrap

deploy:
	sam deploy --guided --no-fail-on-empty-changeset --no-confirm-changeset --profile test --stack-name ${STACK_NAME} --template-file template.yml

delete:
	sam delete --profile test --stack-name ${STACK_NAME}