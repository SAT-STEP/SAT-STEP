#include "../../src/ipasir.h"

#ifdef NDEBUG
#undef NDEBUG
#endif

#include <assert.h>
#include <signal.h>
#include <stdio.h>
#include <unistd.h>

static const int n = 8;

static int ph (int p, int h) {
  assert (0 <= p), assert (p < n + 1);
  assert (0 <= h), assert (h < n);
  return 1 + h * (n+1) + p;
}

// Construct a pigeon hole formula for 'n+1' pigeons in 'n' holes.
//
static void formula (void *solver)
{
  for (int h = 0; h < n; h++)
    for (int p1 = 0; p1 < n + 1; p1++)
      for (int p2 = p1 + 1; p2 < n + 1; p2++)
	ipasir_add (solver, -ph (p1, h)),
	ipasir_add (solver, -ph (p2, h)),
	ipasir_add (solver, 0);

  for (int p = 0; p < n + 1; p++) {
    for (int h = 0; h < n; h++)
      ipasir_add (solver, ph (p, h));
    ipasir_add (solver, 0);
  }
}

typedef struct learner learner;

struct learner {
  void * solver;
  unsigned learned;
};

static void learn (void * ptr, int * clause) {
  learner * learner = ptr;
  for (const int * p = clause; *p; p++)
    ipasir_add (learner->solver, *p);
  ipasir_add (learner->solver, 0);
  learner->learned++;
}

static int terminator (void * ptr) { return * (int*) ptr; }

static int terminate;

static void (*saved)(int);

static void handler (int sig) {
  assert (sig == SIGALRM);
  signal (SIGALRM, saved);
  terminate = 1;
}

static void * solvers[2];
static learner learners[2];

int main () {
  printf ("signature '%s'\n", ipasir_signature ());
  for (int i = 0; i < 2; i++) {
    learners[i].solver = solvers[i] = ipasir_init ();
    ipasir_set_learn (solvers[i], learners + !i, 3, learn);
    formula (solvers[i]);
  }
  unsigned round = 0;
  int active = 0;
  int res = 0;
  for (;;) {
    printf ("round %d active %d imported %u\n",
            ++round, active, learners[active].learned);
    fflush (stdout);
    saved = signal (SIGALRM, handler);
    ualarm (2e4, 0);
    ipasir_set_terminate (solvers[active], &terminate, terminator);
    res = ipasir_solve (solvers[active]);
    if (res) break;
    terminate = 0;
    active = !active;
  }
  for (int i = 0; i < 2; i++)
    ipasir_release (solvers[i]);
  for (int i = 0; i < 2; i++)
    printf ("solver[%d] imported %u clauses\n", i, learners[i].learned);
  return 0;
}
