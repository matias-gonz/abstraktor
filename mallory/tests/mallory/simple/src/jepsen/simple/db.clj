(ns jepsen.simple.db
  (:require [clojure.tools.logging :refer :all]
            [clojure.string :as str]
            [jepsen [control :as c]
                    [db :as db]]
            [jepsen.control.util :as cu]))

(def dir "/opt/fs/simple")
(def bin "app")
(def binary (str dir "/" bin))
(def logfile (str dir "/app.log"))
(def pidfile (str dir "/app.pid"))

(defn install! [test node]
  (let [user (c/exec :whoami)]
    (c/su
     (c/exec :mkdir :-p dir)
     (c/exec :chown user dir)))
  (if-let [pre-built-binary (:binary test)]
    (do (c/upload pre-built-binary binary)
        (c/su (c/exec :chmod :+x binary)))
    (throw (ex-info "--binary path is required for simple app" {}))))

(defn start! [test node]
  (cu/start-daemon! {:logfile logfile
                     :pidfile pidfile
                     :chdir dir}
                    binary))

(defn kill! [test node]
  (cu/stop-daemon! pidfile))

(defn db []
  (reify db/DB
    (setup! [_ test node]
      (install! test node)
      (start! test node))

    (teardown! [_ test node]
      (kill! test node))

    db/LogFiles
    (log-files [_ test node]
      [logfile])

    db/Process
    (start! [_ test node] (start! test node))
    (kill! [_ test node] (kill! test node))))


