(defproject jepsen.simple "0.1.0"
  :description "Jepsen tests for a simple FSM app."
  :dependencies [[org.clojure/clojure "1.11.1"]
                 [jepsen "0.2.7-MEDIATOR-SNAPSHOT"]
                 [clj-http "3.10.1"]]
  :main jepsen.simple
  :jvm-opts ["-Djava.awt.headless=true"
             "-server"]
  :repl-options {:init-ns jepsen.simple})

