#include "../include/types.h"
#include "../include/config.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/shm.h>
#include <sys/time.h>
#include <sys/syscall.h>
#include <pthread.h>

#define CONST_PRIO 0

const u64 BLOCK_EVENT_TYPE = 1;
const u64 FUNC_EVENT_TYPE = 2;
// 3 = packet send; 
// 4 = packet receive
const u64 CONST_EVENT_TYPE = 5;

#pragma pack(8) // 8-byte memory alignment
/** Event entry **/
struct Event
{
  union
  {
    struct
    {
      u16 evtCounter;
      u64 batchNumber;
    };
    struct
    {
      u64 blockEventType; // 1: block
      s64 blockEventTimestamp;
      u64 blockEventID;
      char blockFuncName[64];
      char stateBlockName[64];
    };
    struct
    {
      u64 fevtType; // 2: function
      s64 ftimestamp;
      u64 fevtID;
      char funcName[64];
      char stateFuncName[64];
    };
    struct
    {
      u64 constEventType; // 5: constant
      s64 constEventTimestamp;
      u64 constEventID;
      char constFuncName[64];
      char constEventName[64];
    };
    struct
    {
      u64 evtType; // 3 = packet send; 4 = packet receive
      s64 timestamp;
      u64 evtID; // a mostly-unique identifier for the packet
    };
  };
};

struct Event evtVec[EVT_SIZE];
struct Event *evtVec_ptr = evtVec; // evtID in idx_0: to record evt counter

// For saving AFL feedback
u8 __afl_area_initial[MAP_SIZE];
u8 *__afl_area_ptr = __afl_area_initial;

__thread u32 __afl_prev_loc;

/***
 * instrument block starting point
 ***/
void trigger_block_event(u16 evtID, char* function_name, void** parameters, long long size)
{
  /* find location to record this event */
  u16 loc = __atomic_add_fetch(&evtVec_ptr[0].evtCounter, 1, __ATOMIC_RELAXED);

  /* collect tid and timestamp */
  struct timespec st;
  clock_gettime(CLOCK_MONOTONIC, &st);
  s64 time = st.tv_sec * 1000000000 + st.tv_nsec;

  /* record this event */
  evtVec_ptr[loc].blockEventType = BLOCK_EVENT_TYPE;
  evtVec_ptr[loc].blockEventTimestamp = time;
  evtVec_ptr[loc].blockEventID = evtID;

  u16* com = (u16*)parameters[0];

  int v = 0;

  u16 state = *com;
  char* final_state;
  if(state == 0){
    final_state = "Unavailable";
  } else if (state == 1){
    final_state = "Follower";
  } else if (state == 2){
    final_state = "Candidate";

    if (size == 3){
      void* second_slot = parameters[1]; 

      bool** votes_ptr = (bool**)second_slot;

      bool* votes = *votes_ptr;

      void* third_slot = parameters[2];
      u64* n_voters_ptr = (u64*)third_slot;
      long long n_voters = *n_voters_ptr;
      //long long n_voters = 5;
      size_t half = n_voters / 2;

      for(int i = 0; i < n_voters; i++){
        if(votes[i]){
          v++;
        }
      }
      if (v >= half) {
        final_state = "CandidateVotesInQuorum";
      } else {
        final_state = "CandidateNotVotesInQuorum";
      }
    } else if (size == 9){
      void* second_slot = parameters[1]; 

      bool** votes_ptr = (bool**)second_slot;

      bool* votes = *votes_ptr;

      void* third_slot = parameters[4];
      u64* n_voters_ptr = (u64*)third_slot;
      long long n_voters = *n_voters_ptr;
      //long long n_voters = 5;
      size_t half = n_voters / 2;

      for(int i = 0; i < n_voters; i++){
        if(votes[i]){
          v++;
        }
      }
      if (v >= half) {
        final_state = "CandidateVotesInQuorum";
      } else {
        final_state = "CandidateNotVotesInQuorum";
      }
    }
  } else if (state == 3){
    final_state = "Leader";

    if (size == 7) {
      void* second_slot = parameters[1]; 

      u32* current_term_ptr = (u32*)second_slot;

      u32 current_term = *current_term_ptr;

      void* third_slot = parameters[2];
      u64* commit_index_ptr = (u64*)third_slot;
      u64 commit_index = *commit_index_ptr;

      void* fourth_slot = parameters[3];
      u64* last_log_index_ptr = (u64*)fourth_slot;
      u64 last_log_index = *last_log_index_ptr;

      void* fifth_slot = parameters[4];
      bool* exist_ptr = (bool*)fifth_slot;
      bool existIndex = *exist_ptr;

      void* sixth_slot = parameters[5];
      u64* max_index_quorum_ptr = (u64*)sixth_slot;
      u64 max_index_quorum = *max_index_quorum_ptr;

      void* seventh_slot = parameters[6];
      u64* log_term_max_index_quorum_ptr = (u64*)seventh_slot;
      u64 log_term_max_index_quorum = *log_term_max_index_quorum_ptr;

      bool matching_quorum = /*existIndex && */(log_term_max_index_quorum == current_term);

      bool not_matching_quorum = !matching_quorum;

      bool commit_at_end = (commit_index == last_log_index);

      bool commit_not_at_end = !commit_at_end;

      bool leaderNotMatchingQuorumLogUpdated = not_matching_quorum && commit_at_end;
      bool leaderMatchingQuorumLogUpdated = matching_quorum && commit_at_end;
      bool leaderNotMatchingQuorumNotLogUpdated = not_matching_quorum && commit_not_at_end;
      bool leaderMatchingQuorumNotLogUpdated = matching_quorum && commit_not_at_end;

      if (leaderNotMatchingQuorumLogUpdated) {
        final_state = "LeaderNotMatchingQuorumLogUpdated";
      } else if (leaderMatchingQuorumLogUpdated) {
        final_state = "LeaderMatchingQuorumLogUpdated";
      } else if (leaderNotMatchingQuorumNotLogUpdated) {
        final_state = "LeaderNotMatchingQuorumNotLogUpdated";
      } else if (leaderMatchingQuorumNotLogUpdated) {
        final_state = "LeaderMatchingQuorumNotLogUpdated";
      }
    } else if (size == 9){
      void* second_slot = parameters[2]; 

      u32* current_term_ptr = (u32*)second_slot;

      u32 current_term = *current_term_ptr;

      void* third_slot = parameters[3];
      u64* commit_index_ptr = (u64*)third_slot;
      u64 commit_index = *commit_index_ptr;

      void* fourth_slot = parameters[5];
      u64* last_log_index_ptr = (u64*)fourth_slot;
      u64 last_log_index = *last_log_index_ptr;

      void* fifth_slot = parameters[6];
      bool* exist_ptr = (bool*)fifth_slot;
      bool existIndex = *exist_ptr;

      void* sixth_slot = parameters[7];
      u64* max_index_quorum_ptr = (u64*)sixth_slot;
      u64 max_index_quorum = *max_index_quorum_ptr;

      void* seventh_slot = parameters[8];
      u64* log_term_max_index_quorum_ptr = (u64*)seventh_slot;
      u64 log_term_max_index_quorum = *log_term_max_index_quorum_ptr;

      bool matching_quorum = existIndex && (log_term_max_index_quorum == current_term);

      bool not_matching_quorum = !matching_quorum;

      bool commit_at_end = (commit_index == last_log_index);

      bool commit_not_at_end = !commit_at_end;

      bool leaderNotMatchingQuorumLogUpdated = not_matching_quorum && commit_at_end;
      bool leaderMatchingQuorumLogUpdated = matching_quorum && commit_at_end;
      bool leaderNotMatchingQuorumNotLogUpdated = not_matching_quorum && commit_not_at_end;
      bool leaderMatchingQuorumNotLogUpdated = matching_quorum && commit_not_at_end;

      if (leaderNotMatchingQuorumLogUpdated) {
        final_state = "LeaderNotMatchingQuorumLogUpdated";
      } else if (leaderMatchingQuorumLogUpdated) {
        final_state = "LeaderMatchingQuorumLogUpdated";
      } else if (leaderNotMatchingQuorumNotLogUpdated) {
        final_state = "LeaderNotMatchingQuorumNotLogUpdated";
      } else if (leaderMatchingQuorumNotLogUpdated) {
        final_state = "LeaderMatchingQuorumNotLogUpdated";
      }
    }
  } else {
    final_state = "Unknown";
  }

  strcpy(evtVec_ptr[loc].blockFuncName, function_name);
  strcpy(evtVec_ptr[loc].stateBlockName, final_state);
}

void trigger_func_event(u16 evtID, char* function_name, void** parameters, long long size)
{
  /* find location to record this event */
  u16 loc = __atomic_add_fetch(&evtVec_ptr[0].evtCounter, 1, __ATOMIC_RELAXED);

  /* collect tid and timestamp */
  struct timespec st;
  clock_gettime(CLOCK_MONOTONIC, &st);
  s64 time = st.tv_sec * 1000000000 + st.tv_nsec;

  /* record this event */
  evtVec_ptr[loc].fevtType = FUNC_EVENT_TYPE;
  evtVec_ptr[loc].ftimestamp = time;
  evtVec_ptr[loc].fevtID = evtID;

  u16* com = (u16*)parameters[0];

  int v = 0;

  u16 state = *com;
  char* final_state;
  if(state == 0){
    final_state = "Unavailable";
  } else if (state == 1){
    final_state = "Follower";
  } else if (state == 2){
    final_state = "Candidate";

  // if (size == 3){
  //     void* second_slot = parameters[1]; 

  //     bool** votes_ptr = (bool**)second_slot;

  //     bool* votes = *votes_ptr;

  //     void* third_slot = parameters[2];
  //     u16* n_voters_ptr = (u16*)third_slot;
  //     int n_voters = *n_voters_ptr;
  //     //long long n_voters = 5;
  //     size_t half = n_voters / 2;

  //     for(int i = 0; i < n_voters; i++){
  //       if(votes[i]){
  //         v++;
  //       }
  //     }
  //     if (v >= half) {
  //     final_state = "CandidateVotesInQuorum";
  //   } else {
  //     final_state = "CandidateNotVotesInQuorum";
  //   }
  // }

   
  } else if (state == 3){
    final_state = "Leader";
  } else {
    final_state = "Unknown";
  }
  strcpy(evtVec_ptr[loc].funcName, function_name);
  strcpy(evtVec_ptr[loc].stateFuncName, final_state);
}

void trigger_const_event(u16 evtID, char* function_name, char* const_string)
{
  /* find location to record this event */
  u16 loc = __atomic_add_fetch(&evtVec_ptr[0].evtCounter, 1, __ATOMIC_RELAXED);

  /* collect tid and timestamp */
  struct timespec st;
  clock_gettime(CLOCK_MONOTONIC, &st);
  s64 time = st.tv_sec * 1000000000 + st.tv_nsec;

  /* record this event */
  evtVec_ptr[loc].constEventType = CONST_EVENT_TYPE;
  evtVec_ptr[loc].constEventTimestamp = time;
  evtVec_ptr[loc].constEventID = evtID;

  strcpy(evtVec_ptr[loc].constFuncName, function_name);
  strcpy(evtVec_ptr[loc].constEventName, const_string);
}

/***
 * instrument functions starting point
 ***/
void track_functions(u16 evtID)
{
  /* find location to record this event */
  u16 loc = __atomic_add_fetch(&evtVec_ptr[0].evtCounter, 1, __ATOMIC_RELAXED);

  /* collect tid and timestamp */
  struct timespec st;
  clock_gettime(CLOCK_MONOTONIC, &st);
  s64 time = st.tv_sec * 1000000000 + st.tv_nsec;

  /* record this event */
  evtVec_ptr[loc].fevtType = FUNC_EVENT_TYPE;
  evtVec_ptr[loc].fevtID = evtID;
  evtVec_ptr[loc].ftimestamp = time;
}

void init_shm_dsfuzz()
{
  FILE *fp = NULL;
  fp = fopen("/opt/shm/dsfuzz_shm_id", "r");

  if (fp)
  {
    u8 shm_id_str[4]; // key type: int for C/C++
    if (fscanf(fp, "%s", shm_id_str) != EOF)
    {
      u32 shm_id = atoi(shm_id_str);
      evtVec_ptr = (struct Event *)shmat(shm_id, NULL, 0);
      if (evtVec_ptr == (void *)-1)
      {
        perror("[FAILED] shmat");
        fprintf(stderr, "[!!!] DS shared memory error: subject fails to access shared memory\n");
        _exit(1);
      }
    }
    fclose(fp);
  }
  else
  {
    fprintf(stderr, "[!!!] DS shared memory warning: subject is not running under the coverage server\n");
  }
}

void init_shm_afl()
{
  FILE *fp = NULL;
  fp = fopen("/opt/shm/afl_shm_id", "r");

  if (fp)
  {
    u8 shm_id_str[4]; // key type: int for C/C++
    if (fscanf(fp, "%s", shm_id_str) != EOF)
    {
      u32 shm_id = atoi(shm_id_str);
      __afl_area_ptr = shmat(shm_id, NULL, 0);
      if (__afl_area_ptr == (void *)-1)
      {
        perror("[FAILED] shmat");
        fprintf(stderr, "[!!!] AFL shared memory error: Subject fails to access shared memory\n");
        _exit(1);
      }
    }
    fclose(fp);
  }
  else
  {
    fprintf(stderr, "[!!!] AFL shared memory warning: Subject is not running under the coverage server\n");
  }
}

__attribute__((constructor(CONST_PRIO))) void __afl_auto_init(void)
{
  init_shm_afl();
  init_shm_dsfuzz();
}

/* The following stuff deals with supporting -fsanitize-coverage=trace-pc-guard.
   It remains non-operational in the traditional, plugin-backed LLVM mode.

   The first function (__sanitizer_cov_trace_pc_guard) is called back on every
   edge (as opposed to every basic block). */

void __sanitizer_cov_trace_pc_guard(uint32_t *guard)
{
  __afl_area_ptr[*guard]++;
}

/* Init callback. Populates instrumentation IDs. Note that we're using
   ID of 0 as a special value to indicate non-instrumented bits. That may
   still touch the bitmap, but in a fairly harmless way. */

void __sanitizer_cov_trace_pc_guard_init(uint32_t *start, uint32_t *stop)
{
  u32 inst_ratio = 100;
  u8 *x;

  if (start == stop || *start)
    return;

  x = getenv("AFL_INST_RATIO");
  if (x)
    inst_ratio = atoi(x);

  if (!inst_ratio || inst_ratio > 100)
  {
    fprintf(stderr, "[-] ERROR: Invalid AFL_INST_RATIO (must be 1-100).\n");
    abort();
  }

  /* Make sure that the first element in the range is always set - we use that
     to avoid duplicate calls (which can happen as an artifact of the underlying
     implementation in LLVM). */

  *(start++) = AFL_RR(MAP_SIZE - 1) + 1;

  while (start < stop)
  {

    if (AFL_RR(100) < inst_ratio)
      *start = AFL_RR(MAP_SIZE - 1) + 1;
    else
      *start = 0;

    start++;
  }
}