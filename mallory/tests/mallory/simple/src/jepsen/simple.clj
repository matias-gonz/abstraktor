(ns jepsen.simple
  (:gen-class)
  (:refer-clojure :exclude [test])
  (:require [clojure.tools.logging :refer :all]
            [jepsen [checker :as checker]
             [cli :as cli]
             [generator :as gen]
             [tests :as tests]
             [client :as jclient]]
            [jepsen.os.debian :as debian]
            [jepsen.simple [db :as sdb]
             [client :as scli]]))

(defrecord Client [conn]
  jclient/Client
  (open! [this test node]
    (assoc this :conn (scli/open test (first (:nodes test)))))
  (close! [this test])
  (setup! [this test])
  (teardown! [this test])
  (invoke! [this test op]
    (case (:f op)
      :get-state (assoc op :type :ok :value (scli/get-state conn))
      :next      (assoc op :type :ok :value (scli/next-state! conn)))))

(defn checker-state-cycle []
  (reify checker/Checker
    (check [this test history opts]
      (let [nexts (->> history (filter #(= (:f %) :next)) (filter #(= :ok (:type %))) (map :value))
            expected (cycle ["B" "C" "D" "E" "A"]) 
            ok? (every? true? (map = nexts expected))]
        {:valid? ok? :count (count nexts)}))) )

(defn workload [opts]
  {:client (->Client nil)
   :generator (gen/mix
               [{:type :invoke, :f :next, :value nil}
                {:type :invoke, :f :get-state, :value nil}])
   :checker (checker/compose {:stats (checker/stats)
                              :state (checker-state-cycle)})})

(defn test [opts]
  (merge tests/noop-test
         opts
         {:name "simple-fsm"
          :pure-generators true
          :os debian/os
          :concurrency 1
          :db (sdb/db)
          :client (:client (workload opts))
          :checker (:checker (workload opts))
          :generator (->> (:generator (workload opts))
                          (gen/stagger 1)
                          (gen/time-limit (:time-limit opts)))}))

(def cli-opts
  [["-r" "--rate HZ" "Approximate request rate, in hz" :default 5 :parse-fn parse-long]
   [nil "--time-limit SECS" "Test time limit" :default 10 :parse-fn parse-long]
   ["-b" "--binary BINARY" "Path to pre-built simple app binary" :default "../tests/simple_fsm/fsm_app"]])

(defn -main [& args]
  (cli/run!
   (merge (cli/serve-cmd)
          (cli/single-test-cmd {:test-fn test
                                :opt-spec cli-opts}))
   args))


