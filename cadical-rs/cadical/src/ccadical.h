#ifndef _ccadical_h_INCLUDED
#define _ccadical_h_INCLUDED

/*------------------------------------------------------------------------*/
#include <cstdint>
#ifdef __cplusplus
extern "C" {
#endif
/*------------------------------------------------------------------------*/

#include <stdint.h>

// C wrapper for CaDiCaL's C++ API following IPASIR.

typedef struct CCaDiCaL CCaDiCaL;

const char * ccadical_signature (void);
CCaDiCaL * ccadical_init (void);
void ccadical_release (CCaDiCaL *);

void ccadical_add (CCaDiCaL *, int lit);
void ccadical_assume (CCaDiCaL *, int lit);
int ccadical_solve (CCaDiCaL *);
int ccadical_val (CCaDiCaL *, int lit);
int ccadical_failed (CCaDiCaL *, int lit);

void ccadical_set_terminate (CCaDiCaL *,
  void * state, int (*terminate)(void * state));

void ccadical_set_learn (CCaDiCaL *,
  void * state, int max_length, void (*learn)(void * state, int * clause));

// PAAVO:
void ccadical_set_learn_trail (CCaDiCaL *,
  void * state, void (*trail)(void * state, unsigned long conflict_size, int * conflict_literals, unsigned long propagated_size, int * is_propagated, unsigned long size, int * trail));

double ccadical_process_time (CCaDiCaL *);
double ccadical_real_time (CCaDiCaL *);
double ccadical_max_resident_set_size (CCaDiCaL *);
int64_t ccadical_conflicts (CCaDiCaL *);
int64_t ccadical_learned_clauses (CCaDiCaL *);
int64_t ccadical_learned_literals (CCaDiCaL *);
int64_t ccadical_decisions (CCaDiCaL *);
int64_t ccadical_restarts(CCaDiCaL *);
/*------------------------------------------------------------------------*/

// Non-IPASIR conformant 'C' functions.

void ccadical_set_option (CCaDiCaL *, const char * name, int val);
void ccadical_limit (CCaDiCaL *, const char * name, int limit);
int ccadical_get_option (CCaDiCaL *, const char * name);
void ccadical_print_statistics (CCaDiCaL *);
int64_t ccadical_active (CCaDiCaL *);
int64_t ccadical_irredundant (CCaDiCaL *);
int ccadical_fixed (CCaDiCaL *, int lit);
void ccadical_terminate (CCaDiCaL *);
void ccadical_freeze (CCaDiCaL *, int lit);
int ccadical_frozen (CCaDiCaL *, int lit);
void ccadical_melt (CCaDiCaL *, int lit);
int ccadical_simplify (CCaDiCaL *);

/*------------------------------------------------------------------------*/

// Support legacy names used before moving to more IPASIR conforming names.

#define ccadical_reset ccadical_release
#define ccadical_sat ccadical_solve
#define ccadical_deref ccadical_val

/*------------------------------------------------------------------------*/
#ifdef __cplusplus
}
#endif
/*------------------------------------------------------------------------*/

#endif
