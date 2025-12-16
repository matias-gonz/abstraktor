Buenas! Perdón por el delay, todavía estoy haciendo las slides, estoy siguiendo esta estructura, les parece bien? sacarían/agregarían algo?

1. Portada
2. Agenda (secciones)

EL PROBLEMA
3. Problema detectado
4. Herramientas de testing actuales (Jepsen y Mallory detectan crashes pero no comportamiento)
5. EPAs -> como las epas ayudan a validar la implementacion
6. Ejemplo de EPA (stack u otro)

LA SOLUCION
7. Proposito de abstraktor (generar epas con fuzzing de implementaciones)
8. Arquitectura de Abstraktor -> diagrama de componentes

COMO FUNCIONA (el orden en esta seccion no estoy seguro, seguro lo cambio)
9. Flujo de trabajo/diagrama de secuencia
10. Sistema de anotaciones
11. Instrumentador LLVM
12. Mallory (explicar que es codigo ya hecho y que extendimos)
13. CLI de Abstraktor

DEMO
14. Demo: El problema: codigo del worker con bug
15. Demo: Escenario: mensaje duplicado por delay de red
16. Demo: EPA esperada vs EPA generada

METODOLOGIA
17. Contexto del desarrollo (lafhis, equipo, esto seguramente hayq ue mencionarlo antes)
18. Proceso de desarrollo (agil, iteraciones, prs, etc)
19. Métricas

DESAFIOS
20. Desafios de codigo legacy (tests y documentacion) + estrategias aplicadas
21. Desafios de colaboracion (objetivos distintos) + estrategias aplicadas
22. Articulo aceptado en SERS

CIERRE
23. Trabajos futuros (experimentacion con Raft, mejoras tecnicas)
24. Conclusiones
25. Preguntas
26. Gracias
