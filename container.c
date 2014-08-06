#include <stdio.h>
#include <string.h>
#include <sched.h>
#include <stdlib.h>
#include <unistd.h>
#include <signal.h>
#include <sys/types.h>
#include <sys/mount.h>
#include <sys/prctl.h>
#include <unistd.h>
#include <errno.h>

enum pid1mode_t {
    pid1_exec = 0,
    pid1_wait = 1,
    pid1_waitallchildren = 2,
};

enum extflags_t {
    flag_mkdir = 1,
};

/* Keep in sync with linux.rs */
struct mount {
    char *source;
    char *target;
    char *fstype;
    char *options;
    unsigned long flags;
    unsigned ext_flags;
};

/* Keep in sync with linux.rs */
struct container {
    enum pid1mode_t pid1_mode;
    int pipe_reader;
    int pipe_writer;
    const char *container_root;
    const char *mount_dir;
    int mounts_num;
    struct mount *mounts;
    const char *work_dir;
    char *exec_filename;  // Basically for error message
    int exec_filenames_num;
    char **exec_filenames;  // Real paths to try
    char **exec_args;
    char **exec_environ;
};


inline void check_error(int rc, const char *pattern, const char *val) {
    if(rc < 0) {
        fprintf(stderr, pattern, val, errno, strerror(errno));
        exit(121);
    }
}


void mount_all(int num, struct mount *mounts) {
    int i;
    for(i = 0; i < num; ++i) {
        struct mount *mnt = &mounts[i];
        if(mnt->ext_flags & flag_mkdir) {
            check_error(mkdir(mnt->target, 0755),
               "Can't mkdir %s: (%d) %s\n", mnt->target);
        }
        if(mnt->flags & (MS_BIND | MS_RDONLY) == (MS_BIND | MS_RDONLY)) {
            //  Can bind readonly right away must first just bind
            //  then remount readonly
            int flags1 = mnt->flags & ~(MS_REMOUNT | MS_RDONLY);
            int flags2 = MS_BIND | MS_RDONLY | MS_REMOUNT;
            check_error(mount(mnt->source, mnt->target, mnt->fstype,
                              flags1, NULL),
                "Can't mount %s: (%d) %s\n", mnt->target);
            check_error(mount(mnt->source, mnt->target, mnt->fstype,
                              flags2, NULL),
                "Can't remount ro %s: (%d) %s\n", mnt->target);
        } else {
            check_error(mount(mnt->source, mnt->target, mnt->fstype,
                              mnt->flags, NULL),
                "Can't mount %s: (%d) %s\n", mnt->target);
        }
    }
}

void execute(struct container *cont) {
    int i;
    sigset_t sigset;
    sigemptyset(&sigset);
    sigprocmask(SIG_SETMASK, &sigset, NULL);

    struct sigaction sig_action;
    sig_action.sa_handler = SIG_DFL;
    sig_action.sa_flags = 0;
    sigemptyset(&sig_action.sa_mask);

    for(i = 0 ; i < NSIG ; i++)
        sigaction(i, &sig_action, NULL);

    for(i = 0; i < cont->exec_filenames_num; ++i) {
        execve(cont->exec_filenames[i], cont->exec_args, cont->exec_environ);
    }
    check_error(-1, "Couldn't exec file %s: (%d) %s\n", cont->exec_filename);
    exit(127);
}

void exec_and_wait(struct container *cont) {
    int child_pid, child_status;
    sigset_t sset;
    int sig;
    int exit_code = 0;

    pid_t pid = fork();
    check_error(pid, "Couldn't %s: (%d) %s\n", "fork");
    if(pid == 0) {
        execute(cont);
        exit(121);
    }

    sigfillset(&sset);
    do {
        sigwait(&sset, &sig);
        switch(sig) {
        case SIGCHLD:
            while((child_pid = waitpid(-1, &child_status, WNOHANG)) > 0) {
                if(child_pid == pid) {
                    exit_code = child_status;
                }
            }
            break;
        default:
            kill(pid, sig);
            break;
        }
    } while(kill(pid, 0) == 0);

    // Can't emulate signals (can we?) so use bash-style code translation
    if(WIFEXITED(exit_code)) {
        // Normal exit status
        exit(WEXITSTATUS(exit_code));
    } else {
        exit(128 + WTERMSIG(exit_code));
    }
}

void exec_and_wait_any(struct container *cont) {
    int child_pid, child_status;
    sigset_t sset;
    int sig;
    int exit_code = 0;

    pid_t pid = fork();
    check_error(pid, "Couldn't %s: (%d) %s\n", "fork");
    if(pid == 0) {
        execute(cont);
        exit(121);
    }

    sigfillset(&sset);
    while(1) {
        sigwait(&sset, &sig);
        switch(sig) {
        case SIGCHLD:
            while((child_pid = waitpid(-1, &child_status, WNOHANG)) > 0) {
                if(child_pid == pid) {
                    exit_code = child_status;
                }
            }
            if(child_pid < 0 && errno == ECHILD) {
                exit(exit_code);
            }
            break;
        default:
            kill(-1, sig);
            break;
        }
    };
    exit(exit_code);
}

int _run_container(void *arg) {
    int i, rc;
    char val[1];
    struct container *cont = arg;

    check_error(prctl(PR_SET_PDEATHSIG, SIGKILL, 0, 0, 0),
        "Can't set %s: (%d) %s", "DEATHSIG");

    do {
        rc = read(cont->pipe_reader, val, 1);
    } while(rc < 0 && (errno == EINTR || errno == EAGAIN));
    check_error(rc,
        "Can't read from %s: (%d) %s\n", "pipe");
    close(cont->pipe_reader);
    close(cont->pipe_writer);

    mount_all(cont->mounts_num, cont->mounts);

    check_error(chdir(cont->mount_dir),
        "Can't set working directory to %s: (%d) %s\n", cont->mount_dir);
    check_error(chroot(cont->mount_dir),
        "Can't change root to %s: (%d) %s\n", cont->mount_dir);
    check_error(chdir(cont->work_dir),
        "Can't set working directory to %s: (%d) %s\n", cont->work_dir);

    switch(cont->pid1_mode) {
    case pid1_exec:
        execute(cont);
        break;
    case pid1_wait:
        exec_and_wait(cont);
        break;
    case pid1_waitallchildren:
        exec_and_wait_any(cont);
        break;
    default:
        fprintf(stderr, "Internal Error: Wrong pid1mode %s\n", cont->pid1_mode);
        exit(121);
    }
}




pid_t fork_to_container(int flags, struct container *container) {

    size_t stack_size = sysconf(_SC_PAGESIZE);
    void *stack = alloca(stack_size);

    pid_t pid = clone(_run_container,
        stack + stack_size,
        flags|SIGCHLD,
        container);

    return pid;
}
