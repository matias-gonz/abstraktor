/*
  Copyright 2013 Google LLC All rights reserved.

  Licensed under the Apache License, Version 2.0 (the "License");
  you may not use this file except in compliance with the License.
  You may obtain a copy of the License at:

    http://www.apache.org/licenses/LICENSE-2.0

  Unless required by applicable law or agreed to in writing, software
  distributed under the License is distributed on an "AS IS" BASIS,
  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
  See the License for the specific language governing permissions and
  limitations under the License.
*/


/*
   american fuzzy lop - type definitions and minor macros
   ------------------------------------------------------

   Written and maintained by Michal Zalewski <lcamtuf@google.com>
*/

#ifndef _HAVE_TYPES_H
#define _HAVE_TYPES_H

#include <stdint.h>
#include <stdlib.h>
#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>

typedef uint8_t  u8;
typedef uint16_t u16;
typedef uint32_t u32;
typedef uint64_t u64;

/*

   Ugh. There is an unintended compiler / glibc #include glitch caused by
   combining the u64 type an %llu in format strings, necessitating a workaround.

   In essence, the compiler is always looking for 'unsigned long long' for %llu.
   On 32-bit systems, the u64 type (aliased to uint64_t) is expanded to
   'unsigned long long' in <bits/types.h>, so everything checks out.

   But on 64-bit systems, it is #ifdef'ed in the same file as 'unsigned long'.
   Now, it only happens in circumstances where the type happens to have the
   expected bit width, *but* the compiler does not know that... and complains
   about 'unsigned long' being unsafe to pass to %llu.

 */

typedef int8_t   s8;
typedef int16_t  s16;
typedef int32_t  s32;
typedef int64_t  s64;

#ifndef MIN
#  define MIN(_a,_b) ((_a) > (_b) ? (_b) : (_a))
#  define MAX(_a,_b) ((_a) > (_b) ? (_a) : (_b))
#endif /* !MIN */

#define SWAP16(_x) ({ \
    u16 _ret = (_x); \
    (u16)((_ret << 8) | (_ret >> 8)); \
  })

#define SWAP32(_x) ({ \
    u32 _ret = (_x); \
    (u32)((_ret << 24) | (_ret >> 24) | \
          ((_ret << 8) & 0x00FF0000) | \
          ((_ret >> 8) & 0x0000FF00)); \
  })

#ifdef AFL_LLVM_PASS
#  define AFL_R(x) (random() % (x))
#else
#  define AFL_RR(x) (random() % (x))
#endif /* ^AFL_LLVM_PASS */

#define STRINGIFY_INTERNAL(x) #x
#define STRINGIFY(x) STRINGIFY_INTERNAL(x)

#define MEM_BARRIER() \
  __asm__ volatile("" ::: "memory")

#define likely(_x)   __builtin_expect(!!(_x), 1)
#define unlikely(_x)  __builtin_expect(!!(_x), 0)

/**
 * Size of human-readable error message buffers.
 */
 #define RAFT_ERRMSG_BUF_SIZE 256

 typedef unsigned long long raft_id;
 
 /**
  * Hold the value of a raft term. Guaranteed to be at least 64-bit long.
  */
 typedef unsigned long long raft_term;
 
 /**
  * Hold the value of a raft entry index. Guaranteed to be at least 64-bit long.
  */
 typedef unsigned long long raft_index;
 
 /**
  * Hold a time value expressed in milliseconds since the epoch.
  */
 typedef unsigned long long raft_time;
 
 /**
  * A data buffer.
  */
 struct raft_buffer
 {
     void *base; /* Pointer to the buffer data. */
     size_t len; /* Length of the buffer. */
 };
 
 /**
  * Server role codes.
  */
 enum {
     RAFT_STANDBY, /* Replicate log, does not participate in quorum. */
     RAFT_VOTER,   /* Replicate log, does participate in quorum. */
     RAFT_SPARE    /* Does not replicate log, or participate in quorum. */
 };
 
 /**
  * Hold information about a single server in the cluster configuration.
  * WARNING: This struct is encoded/decoded, be careful when adapting it.
  */
 struct raft_server
 {
     raft_id id;    /* Server ID, must be greater than zero. */
     char *address; /* Server address. User defined. */
     int role;      /* Server role. */
 };
 
 /**
  * Hold information about all servers currently part of the cluster.
  * WARNING: This struct is encoded/decoded, be careful when adapting it.
  */
 struct raft_configuration
 {
     struct raft_server *servers; /* Array of servers member of the cluster. */
     unsigned n;                  /* Number of servers in the array. */
 };

 struct raft_entry
{
    raft_term term;         /* Term in which the entry was created. */
    unsigned short type;    /* Type (FSM command, barrier, config change). */
    struct raft_buffer buf; /* Entry data. */
    void *batch;            /* Batch that buf's memory points to, if any. */
};

struct raft_request_vote
{
    int version;
    raft_term term;            /* Candidate's term. */
    raft_id candidate_id;      /* ID of the server requesting the vote. */
    raft_index last_log_index; /* Index of candidate's last log entry. */
    raft_index last_log_term;  /* Term of log entry at last_log_index. */
    bool disrupt_leader;       /* True if current leader should be discarded. */
    bool pre_vote;             /* True if this is a pre-vote request. */
};
#define RAFT_REQUEST_VOTE_VERSION 2

struct raft_request_vote_result
{
    int version;
    raft_term term;    /* Receiver's current term (candidate updates itself). */
    bool vote_granted; /* True means candidate received vote. */
    bool pre_vote;     /* The response to a pre-vote RequestVote or not. */
};
#define RAFT_REQUEST_VOTE_RESULT_VERSION 2

struct raft_append_entries
{
    int version;
    raft_term term;             /* Leader's term. */
    raft_index prev_log_index;  /* Index of log entry preceeding new ones. */
    raft_term prev_log_term;    /* Term of entry at prev_log_index. */
    raft_index leader_commit;   /* Leader's commit index. */
    struct raft_entry *entries; /* Log entries to append. */
    unsigned n_entries;         /* Size of the log entries array. */
};
#define RAFT_APPEND_ENTRIES_VERSION 0

struct raft_append_entries_result
{
    int version;
    raft_term term;            /* Receiver's current_term. */
    raft_index rejected;       /* If non-zero, the index that was rejected. */
    raft_index last_log_index; /* Receiver's last log entry index, as hint. */
};
#define RAFT_APPEND_ENTRIES_RESULT_VERSION 0

struct raft_install_snapshot
{
    int version;
    raft_term term;                 /* Leader's term. */
    raft_index last_index;          /* Index of last entry in the snapshot. */
    raft_term last_term;            /* Term of last_index. */
    struct raft_configuration conf; /* Config as of last_index. */
    raft_index conf_index;          /* Commit index of conf. */
    struct raft_buffer data;        /* Raw snapshot data. */
};
#define RAFT_INSTALL_SNAPSHOT_VERSION 0

struct raft_timeout_now
{
    int version;
    raft_term term;            /* Leader's term. */
    raft_index last_log_index; /* Index of leader's last log entry. */
    raft_index last_log_term;  /* Term of log entry at last_log_index. */
};
#define RAFT_TIMEOUT_NOW_VERSION 0

/**
 * Type codes for RPC messages.
 */
enum {
    RAFT_IO_APPEND_ENTRIES = 1,
    RAFT_IO_APPEND_ENTRIES_RESULT,
    RAFT_IO_REQUEST_VOTE,
    RAFT_IO_REQUEST_VOTE_RESULT,
    RAFT_IO_INSTALL_SNAPSHOT,
    RAFT_IO_TIMEOUT_NOW
};

struct raft_message
{
    unsigned short type;        /* RPC type code. */
    raft_id server_id;          /* ID of sending or destination server. */
    const char *server_address; /* Address of sending or destination server. */
    union {                     /* Type-specific data */
        struct raft_request_vote request_vote;
        struct raft_request_vote_result request_vote_result;
        struct raft_append_entries append_entries;
        struct raft_append_entries_result append_entries_result;
        struct raft_install_snapshot install_snapshot;
        struct raft_timeout_now timeout_now;
    };
};

struct raft_snapshot
{
    /* Index and term of last entry included in the snapshot. */
    raft_index index;
    raft_term term;

    /* Last committed configuration included in the snapshot, along with the
     * index it was committed at. */
    struct raft_configuration configuration;
    raft_index configuration_index;

    /* Content of the snapshot. When a snapshot is taken, the user FSM can fill
     * the bufs array with more than one buffer. When a snapshot is restored,
     * there will always be a single buffer. */
    struct raft_buffer *bufs;
    unsigned n_bufs;
};

/**
 * Asynchronous request to send an RPC message.
 */
struct raft_io_send;
typedef void (*raft_io_send_cb)(struct raft_io_send *req, int status);
struct raft_io_send
{
    void *data;         /* User data */
    raft_io_send_cb cb; /* Request callback */
};

/**
 * Asynchronous request to store new log entries.
 */
struct raft_io_append;
typedef void (*raft_io_append_cb)(struct raft_io_append *req, int status);
struct raft_io_append
{
    void *data;           /* User data */
    raft_io_append_cb cb; /* Request callback */
};

/**
 * Asynchronous request to store a new snapshot.
 */
struct raft_io_snapshot_put;
typedef void (*raft_io_snapshot_put_cb)(struct raft_io_snapshot_put *req,
                                        int status);
struct raft_io_snapshot_put
{
    void *data;                 /* User data */
    raft_io_snapshot_put_cb cb; /* Request callback */
};

/**
 * Asynchronous request to load the most recent snapshot available.
 */
struct raft_io_snapshot_get;
typedef void (*raft_io_snapshot_get_cb)(struct raft_io_snapshot_get *req,
                                        struct raft_snapshot *snapshot,
                                        int status);
struct raft_io_snapshot_get
{
    void *data;                 /* User data */
    raft_io_snapshot_get_cb cb; /* Request callback */
};

/**
 * Asynchronous work request.
 */
struct raft_io_async_work;
typedef int (*raft_io_async_work_fn)(struct raft_io_async_work *req);
typedef void (*raft_io_async_work_cb)(struct raft_io_async_work *req,
                                      int status);
struct raft_io_async_work
{
    void *data;                 /* User data */
    raft_io_async_work_fn work; /* Function to run async from the main loop */
    raft_io_async_work_cb cb;   /* Request callback */
};

/**
 * Customizable tracer, for debugging purposes.
 */
struct raft_tracer
{
    /**
     * Implementation-defined state object.
     */
    void *impl;

    /**
     * Whether this tracer should emit messages.
     */
    bool enabled;

    /**
     * Emit the given trace message, possibly decorating it with the provided
     * metadata.
     */
    void (*emit)(struct raft_tracer *t,
                 const char *file,
                 int line,
                 const char *message);
};

struct raft_io; /* Forward declaration. */

typedef void (*raft_io_tick_cb)(struct raft_io *io);


typedef void (*raft_io_recv_cb)(struct raft_io *io, struct raft_message *msg);

typedef void (*raft_io_close_cb)(struct raft_io *io);


struct raft_io
{
    int version; /* 1 or 2 */
    void *data;
    void *impl;
    char errmsg[RAFT_ERRMSG_BUF_SIZE];
    int (*init)(struct raft_io *io, raft_id id, const char *address);
    void (*close)(struct raft_io *io, raft_io_close_cb cb);
    int (*load)(struct raft_io *io,
                raft_term *term,
                raft_id *voted_for,
                struct raft_snapshot **snapshot,
                raft_index *start_index,
                struct raft_entry *entries[],
                size_t *n_entries);
    int (*start)(struct raft_io *io,
                 unsigned msecs,
                 raft_io_tick_cb tick,
                 raft_io_recv_cb recv);
    int (*bootstrap)(struct raft_io *io, const struct raft_configuration *conf);
    int (*recover)(struct raft_io *io, const struct raft_configuration *conf);
    int (*set_term)(struct raft_io *io, raft_term term);
    int (*set_vote)(struct raft_io *io, raft_id server_id);
    int (*send)(struct raft_io *io,
                struct raft_io_send *req,
                const struct raft_message *message,
                raft_io_send_cb cb);
    int (*append)(struct raft_io *io,
                  struct raft_io_append *req,
                  const struct raft_entry entries[],
                  unsigned n,
                  raft_io_append_cb cb);
    int (*truncate)(struct raft_io *io, raft_index index);
    int (*snapshot_put)(struct raft_io *io,
                        unsigned trailing,
                        struct raft_io_snapshot_put *req,
                        const struct raft_snapshot *snapshot,
                        raft_io_snapshot_put_cb cb);
    int (*snapshot_get)(struct raft_io *io,
                        struct raft_io_snapshot_get *req,
                        raft_io_snapshot_get_cb cb);
    raft_time (*time)(struct raft_io *io);
    int (*random)(struct raft_io *io, int min, int max);
    /* Field(s) below added since version 2. */
    int (*async_work)(struct raft_io *io,
                      struct raft_io_async_work *req,
                      raft_io_async_work_cb cb);
};

struct raft_fsm
{
    int version; /* 1, 2 or 3 */
    void *data;
    int (*apply)(struct raft_fsm *fsm,
                 const struct raft_buffer *buf,
                 void **result);
    int (*snapshot)(struct raft_fsm *fsm,
                    struct raft_buffer *bufs[],
                    unsigned *n_bufs);
    int (*restore)(struct raft_fsm *fsm, struct raft_buffer *buf);
    /* Fields below added since version 2. */
    int (*snapshot_finalize)(struct raft_fsm *fsm,
                             struct raft_buffer *bufs[],
                             unsigned *n_bufs);
    /* Fields below added since version 3. */
    int (*snapshot_async)(struct raft_fsm *fsm,
                          struct raft_buffer *bufs[],
                          unsigned *n_bufs);
};

/**
 * State codes.
 */
enum { RAFT_UNAVAILABLE, RAFT_FOLLOWER, RAFT_CANDIDATE, RAFT_LEADER };

struct raft_progress;

struct raft; /* Forward declaration. */


typedef void (*raft_close_cb)(struct raft *raft);

struct raft_change;   /* Forward declaration */
struct raft_transfer; /* Forward declaration */

struct raft_log;

/**
 * Hold and drive the state of a single raft server in a cluster.
 * When replacing reserved fields in the middle of this struct, you MUST use a
 * type with the same size and alignment requirements as the original type.
 */
struct raft
{
    void *data;                 /* Custom user data. */
    struct raft_tracer *tracer; /* Tracer implementation. */
    struct raft_io *io;         /* Disk and network I/O implementation. */
    struct raft_fsm *fsm;       /* User-defined FSM to apply commands to. */
    raft_id id;                 /* Server ID of this raft instance. */
    char *address;              /* Server address of this raft instance. */

    /*
     * Cache of the server's persistent state, updated on stable storage before
     * responding to RPCs (Figure 3.1).
     */
    raft_term current_term; /* Latest term server has seen. */
    raft_id voted_for;      /* Candidate that received vote in current term. */
    struct raft_log *log;   /* Log entries. */

    /*
     * Current membership configuration (Chapter 4).
     *
     * At any given moment the current configuration can be committed or
     * uncommitted.
     *
     * If a server is voting, the log entry with index 1 must always contain the
     * first committed configuration.
     *
     * The possible scenarios are:
     *
     * 1. #configuration_index and #configuration_uncommitted_index are both
     *    zero. This should only happen when a brand new server starts joining a
     *    cluster and is waiting to receive log entries from the current
     *    leader. In this case #configuration and #configuration_previous
     *    must be empty and have no servers.
     *
     * 2. #configuration_index is non-zero and #configuration_uncommitted_index
     *    is zero. In this case the content of #configuration must match the one
     *    of the log entry at #configuration_index.
     *
     * 3. #configuration_index and #configuration_uncommitted_index are both
     *    non-zero, with the latter being greater than the former. In this case
     *    the content of #configuration must match the one of the log entry at
     *    #configuration_uncommitted_index.
     *
     * 4. In case the previous - committed - configuration can no longer be
     *    found in the log e.g. after truncating the log when taking or
     *    installing a snapshot, `configuration_previous` will contain a copy
     *    of it.
     */
    struct raft_configuration configuration;
    struct raft_configuration configuration_previous;
    raft_index configuration_index;
    raft_index configuration_uncommitted_index;

    /*
     * Election timeout in milliseconds (default 1000).
     *
     * From 3.4:
     *
     *   Raft uses a heartbeat mechanism to trigger leader election. When
     *   servers start up, they begin as followers. A server remains in follower
     *   state as long as it receives valid RPCs from a leader or
     *   candidate. Leaders send periodic heartbeats (AppendEntries RPCs that
     *   carry no log entries) to all followers in order to maintain their
     *   authority. If a follower receives no communication over a period of
     *   time called the election timeout, then it assumes there is no viable
     *   leader and begins an election to choose a new leader.
     *
     * This is the baseline value and will be randomized between 1x and 2x.
     *
     * See raft_change_election_timeout() to customize the value of this
     * attribute.
     */
    unsigned election_timeout;

    /*
     * Heartbeat timeout in milliseconds (default 100). This is relevant only
     * for when the raft instance is in leader state: empty AppendEntries RPCs
     * will be sent if this amount of milliseconds elapses without any
     * user-triggered AppendEntries RCPs being sent.
     *
     * From Figure 3.1:
     *
     *   [Leaders] Send empty AppendEntries RPC during idle periods to prevent
     *   election timeouts.
     */
    unsigned heartbeat_timeout;

    /*
     * When the leader sends an InstallSnapshot RPC to a follower it will
     * consider the RPC as failed after this timeout and retry.
     */
    unsigned install_snapshot_timeout;

    /*
     * The fields below hold the part of the server's volatile state which is
     * always applicable regardless of the whether the server is follower,
     * candidate or leader (Figure 3.1). This state is rebuilt automatically
     * after a server restart.
     */
    raft_index commit_index; /* Highest log entry known to be committed */
    raft_index last_applied; /* Highest log entry applied to the FSM */
    raft_index last_stored;  /* Highest log entry persisted on disk */

    /*
     * Current server state of this raft instance, along with a union defining
     * state-specific values.
     */
    unsigned short state;
    union {
        struct /* Follower */
        {
            unsigned randomized_election_timeout; /* Timer expiration. */
            struct                                /* Current leader info. */
            {
                raft_id id;
                char *address;
            } current_leader;
            uint64_t reserved[8]; /* Future use */
        } follower_state;
        struct
        {
            unsigned randomized_election_timeout; /* Timer expiration. */
            bool *votes;                          /* Vote results. */
            bool disrupt_leader;                  /* For leadership transfer */
            bool in_pre_vote;                     /* True in pre-vote phase. */
            uint64_t reserved[8];                 /* Future use */
        } candidate_state;
        struct
        {
            struct raft_progress *progress; /* Per-server replication state. */
            struct raft_change *change;     /* Pending membership change. */
            raft_id promotee_id;            /* ID of server being promoted. */
            unsigned short round_number;    /* Current sync round. */
            raft_index round_index;         /* Target of the current round. */
            raft_time round_start;          /* Start of current round. */
            void *requests[2];              /* Outstanding client requests. */
            uint64_t reserved[8];           /* Future use */
        } leader_state;
    };

    /* Election timer start.
     *
     * This timer has different purposes depending on the state. Followers
     * convert to candidate after the randomized election timeout has elapsed
     * without leader contact. Candidates start a new election after the
     * randomized election timeout has elapsed without a winner. Leaders step
     * down after the election timeout has elapsed without contacting a majority
     * of voting servers. */
    raft_time election_timer_start;

    /* In-progress leadership transfer request, if any. */
    struct raft_transfer *transfer;

    /*
     * Information about the last snapshot that was taken (if any).
     */
    struct
    {
        unsigned threshold;              /* N. of entries before snapshot */
        unsigned trailing;               /* N. of trailing entries to retain */
        struct raft_snapshot pending;    /* In progress snapshot */
        struct raft_io_snapshot_put put; /* Store snapshot request */
        uint64_t reserved[8];            /* Future use */
    } snapshot;

    /*
     * Callback to invoke once a close request has completed.
     */
    raft_close_cb close_cb;

    /*
     * Human-readable message providing diagnostic information about the last
     * error occurred.
     */
    char errmsg[RAFT_ERRMSG_BUF_SIZE];

    /* Whether to use pre-vote to avoid disconnected servers disrupting the
     * current leader, as described in 4.2.3 and 9.6. */
    bool pre_vote;

    /* Limit how long to wait for a stand-by to catch-up with the log when its
     * being promoted to voter. */
    unsigned max_catch_up_rounds;
    unsigned max_catch_up_round_duration;

    /* Future extensions */
    uint64_t reserved[32];
};

#endif /* ! _HAVE_TYPES_H */
