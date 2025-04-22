docker-up service:
	echo Composing: {{service}}
	sudo docker compose -f docker-compose.yml up -d --build {{service}}

docker-down service:
	sudo docker compose down {{service}}