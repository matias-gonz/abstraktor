#include "recv_append_entries_result.h"
#include "assert.h"
#include "configuration.h"
#include "tracing.h"
#include "recv.h"
#include "replication.h"
#include "progress.h"
#include "log.h"

#define tracef(...) Tracef(r->tracer, __VA_ARGS__)

// ABSTRAKTOR_FUNC: r->19, r->20->1, r->6, r->16
int recvAppendEntriesResult(struct raft *r,
                            const raft_id id,
                            const char *address,
                            const struct raft_append_entries_result *result)
{
    // ABSTRAKTOR_BLOCK_EVENT: n_voters
    size_t n_voters = configurationVoterCount(&r->configuration);
    (void)n_voters; /* Supress unused variable warning */
    int match;
    const struct raft_server *server;
    int rv;

    assert(r != NULL);
    assert(id > 0);
    assert(address != NULL);
    assert(result != NULL);

    tracef("self:%llu from:%llu@%s last_log_index:%llu rejected:%llu term:%llu",
            r->id, id, address, result->last_log_index, result->rejected, result->term);

    if (r->state != RAFT_LEADER) {
        tracef("local server is not leader -> ignore");
        return 0;
    }

    raft_index log;
    bool exists;
    raft_index max;
    raft_term logTerm;

    if (r->state == RAFT_LEADER) {
        // ABSTRAKTOR_BLOCK_EVENT: log
        log = logLastIndex(r->log); 
        (void)log;

        // ABSTRAKTOR_BLOCK_EVENT: exists
        exists = progressTestExistsOneIndexQuorum(r);
        (void)exists;

        // ABSTRAKTOR_BLOCK_EVENT: max
        max = progressTestGetMaxIndexQuorum(r);
        (void)max;

        // ABSTRAKTOR_BLOCK_EVENT: logTerm END
        logTerm = exists ? logTermOf(r->log, max) : 0;
        (void)logTerm;
    }

    rv = recvEnsureMatchingTerms(r, result->term, &match);
    if (rv != 0) {
        return rv;
    }

    if (match < 0) {
        tracef("local term is higher -> ignore ");
        return 0;
    }

    /* If we have stepped down, abort here.
     *
     * From Figure 3.1:
     *
     *   [Rules for Servers] All Servers: If RPC request or response contains
     *   term T > currentTerm: set currentTerm = T, convert to follower.
     */
    if (match > 0) {
        assert(r->state == RAFT_FOLLOWER);
        return 0;
    }

    assert(result->term == r->current_term);

    /* Ignore responses from servers that have been removed */
    //server->id, r->commit_index, raft_log_len(&r->log), r->current_term);

    server = configurationGet(&r->configuration, id);
    if (server == NULL) {
        tracef("unknown server -> ignore");
        return 0;
    }

    /* Update the progress of this server, possibly sending further entries. */
    rv = replicationUpdate(r, server, result);
    if (rv != 0) {
        return rv;
    }

    return 0;
}

#undef tracef
