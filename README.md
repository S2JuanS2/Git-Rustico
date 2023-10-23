# 23C2-Rusteam-Visionary
Repo for Rust Taller De Programacion 1 FIUBA


# Proyecto de Cliente y Servidor Git

Este es un proyecto en Rust que consta de un cliente y un servidor Git.

## Requisitos

Asegúrate de tener Rust y Cargo instalados en tu sistema. Puedes obtenerlos en [rust-lang.org](https://www.rust-lang.org/).

## Configuración

Antes de ejecutar el proyecto, es necesario configurar ciertos parámetros, como el archivo de configuración. Asegúrate de tener un archivo de configuración válido en la ubicación adecuada.

## Ejecución del Servidor

Para ejecutar el servidor, utiliza el siguiente comando:

```bash
cargo run --bin server -- ./gitconfigserver
```

Esto iniciará el servidor y estará listo para atender las solicitudes de los clientes. Asegúrate de configurar el archivo "path/config" con la información necesaria, incluyendo la dirección IP y el puerto.

## Ejecución del Cliente

Para ejecutar el cliente, utiliza el siguiente comando:

```bash
cargo run --bin client -- ./gitconfigclient
```

El cliente se conectará al servidor utilizando la información proporcionada en el archivo "path/config" y podrá realizar las operaciones disponibles. Asegúrate de que el archivo "path/config" esté configurado correctamente con la dirección IP y el puerto correspondientes.

## Uso

A continuación, se pueden agregar detalles sobre cómo utilizar el servidor y el cliente, incluyendo ejemplos de comandos y opciones disponibles.

## Contribuciones

Si deseas contribuir a este proyecto, por favor consulta nuestras pautas de contribución y realiza un pull request.

## Problemas y Soporte

Si encuentras algún problema o tienes alguna pregunta, por favor crea un issue en este repositorio.

## Licencia

Este proyecto está licenciado bajo la Licencia MIT - consulta el archivo [LICENSE](LICENSE) para más detalles.
