<h1 align="center">Actix JWT API</h1>
<p align="center">
    <img src="https://github.com/user-attachments/assets/322c45a8-5e6e-4581-8042-5f2433263586" width="300px" height="300px"  alt="Actix Mangekyou Sharingan theme Logo" />
</p>

## Descrição
Esse é um projeto simples de API com autenticação sem estado via Json Web Token (JWT). Desenvolvido em Rust, usando o framework Actix e o ORM Diesel.
# Como rodar o projeto
## Pré-requisitos
- Rustc v1.83
- VS Code
- Extensões do VS Code:
    - Rust Analyzer
    - DotENV
    - Cargo
    - Omni Theme (Opcional)
    - Dependi
- Docker
- PgAdmin 4 (Opcional)
- Postman

## Rodando o Projeto
- Clone o repositório usando git clone:
    ```shell
    git clone https://github.com/henrqueDev/actix-jwt-api.git
    ```

- Copiar o .env.example para criar um arquivo .env na pasta raíz do projeto, e preencher conforme a necessidade as variáveis de ambiente
- Subir os containers com docker compose (Já realiza automaticamente migrações no container do banco de dados PostgreSQL):
  ```shell
  docker compose up --build -d
  ```
- Testar os endpoints disponiveis com o Postman
