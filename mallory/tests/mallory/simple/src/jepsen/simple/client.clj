(ns jepsen.simple.client
  (:require [clojure.edn :as edn]
            [clojure.string :as str]
            [slingshot.slingshot :refer [try+ throw+]]
            [clj-http.client :as http]))

(defn endpoint [node]
  (str "http://" (name node) ":" 8080))

(defn open [test node]
  {:endpoint (endpoint node)
   :request-timeout 2000})

(defn request [conn method path]
  (let [url (str (:endpoint conn) path)
        timeout (:request-timeout conn)
        body (str (:body (http/request {:method method
                                        :url url
                                        :socket-timeout timeout
                                        :connection-timeout timeout})))]
    (if (str/includes? body "Error")
      (throw+ {:msg body})
      (edn/read-string body))))

(defn get-state [conn]
  (request conn "GET" "/state"))

(defn next-state! [conn]
  (request conn "POST" "/next"))


