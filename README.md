<h1 align="center">Actix JWT API</h1>
<p align="center">
    <img src="https://github.com/user-attachments/assets/322c45a8-5e6e-4581-8042-5f2433263586" width="300px" height="300px"  alt="Actix Mangekyou Sharingan theme Logo" />
</p>

## Descrição
Esse é um projeto simples de API com autenticação sem estado via Json Web Token (JWT) com 2FA opcional. Desenvolvido em Rust, usando o framework Actix e o ORM Diesel.
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
- Dispositivos com horários sincronizados com o Tempo Universal Coordenado (UTC) (com ou sem fuso horário definido)

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

## Testes de Integração
- Entrar na linha de comando do Container da API (actix_jwt_api por padrão)
    ```shell
    docker exec -it actix_jwt_api bash
    ```

- Rodar a suite de testes:
    ```shell
    cargo test _ -- test_env --show-output
    ```

## Notas sobre middleware de proteção contra Brute Force

- A implementação da proteção é simples, prevenindo ataques de força bruta em:
    - Credenciais de Usuários
    - Códigos de One Timed Password (2FA)
    - Chaves de codificação da API

- Ao rodar a API com docker compose, a rede em modo bridge, ou até mesmo rodando em modo host, a aplicação não é capaz de filtrar os ip's das requisições HTTP corretamente.

    - Uma alternativa para esse problema é retirar a API do docker-compose e rodar o binário diretamente no terminal (ou configurar para rodar como um serviço de sistema linux junto com o Supervisor)

- Os IP's suspeitos de ataques de força bruta ficarão com os acessos à API bloqueados por 5 horas.