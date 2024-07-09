# 23C2-Rusteam-Visionary
Repo for Rust Taller De Programacion 1 FIUBA


# Proyecto de Cliente y Servidor Git

Este es un proyecto en Rust que consta de un cliente y un servidor Git.

## Requisitos

Asegúrate de tener Rust y Cargo instalados en tu sistema. Puedes obtenerlos en [rust-lang.org](https://www.rust-lang.org/).

## Configuración

Antes de ejecutar el proyecto, es necesario configurar ciertos parámetros, como el archivo de configuración. Asegúrate de tener un archivo de configuración válido en la ubicación adecuada.

## Archivo de Configuración del Cliente

El archivo de configuración del cliente debe tener el siguiente formato:

```
    name=VBJ
    email=VJM@visionary.com
    path_log=./client.log
    ip=127.0.0.1
    port=9418
    src=client_root
```
Explicación de los campos:

- name: Nombre del cliente.
- email: Correo electrónico del cliente.
- path_log: Ruta al archivo de log del cliente.
- ip: Dirección IP del servidor al que se conectará el cliente.
- port: Puerto en el que el cliente se conectará al servidor.
- src: Directorio raíz del cliente donde se almacenarán los datos.

## Archivo de Configuración del Servidor

El archivo de configuración del servidor debe tener el siguiente formato:

```
    name=Servercito
    email=servercito@tu.servercito.com
    path_log=./history.log
    ip=127.0.0.1
    port=9418
    src=./server_root
```

Explicación de los campos:

- name: Nombre del servidor.
- email: Correo electrónico del servidor.
- path_log: Ruta al archivo de log del servidor.
- ip: Dirección IP en la que el servidor escuchará las conexiones.
- port: Puerto en el que el servidor aceptará conexiones.
- src: Directorio raíz del servidor donde se almacenarán los datos del repositorio.


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
