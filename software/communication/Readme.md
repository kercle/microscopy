# Communication module for microscopy software

This module specifies the communication protocols between different components of the microscopy software, in particular the firmware communication via UART and the WebSocket protocol for the frontend and compute nodes.

## WebSocket Protocol

To rebuild the TS bindings, run

```bash
just export_bindings
```

The generated bindings are located in `software/frontend/src/lib/bindings`.