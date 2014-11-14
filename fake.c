#define _GNU_SOURCE
#include <dlfcn.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <sys/mount.h>
#include <fcntl.h>
#include <unistd.h>
#include <fcntl.h>
#include <string.h>
#include <stdlib.h>

inline static void libfake_log(const char *val) {
    const char *trace = getenv("LIBFAKE_TRACE");
    if(trace && *trace) {
        write(2, val, strlen(val));
    }
}

int __xmknod(int __ver, const char *__path, __mode_t __mode, __dev_t *dev) {
    libfake_log("VAGGA LIBFAKE: __xmknod ignored\n");
    return 0;
}

int mknod(const char *pathname, mode_t mode, dev_t dev) {
    libfake_log("VAGGA LIBFAKE: mknod ignored\n");
    return 0;
}

int mknodat(int dirfd, const char *pathname, mode_t mode, dev_t dev) {
    libfake_log("VAGGA LIBFAKE: mknodat ignored\n");
    return 0;
}

int chown(const char *pathname, uid_t owner, gid_t group) {
    libfake_log("VAGGA LIBFAKE: chown ignored\n");
    return 0;
}

int fchown(int fd, uid_t owner, gid_t group) {
    libfake_log("VAGGA LIBFAKE: fchown ignored\n");
    return 0;
}

int lchown(const char *pathname, uid_t owner, gid_t group) {
    libfake_log("VAGGA LIBFAKE: lchown ignored\n");
    return 0;
}

int fchownat(int dirfd, const char *pathname,
            uid_t owner, gid_t group, int flags) {
    libfake_log("VAGGA LIBFAKE: fchownat ignored\n");
    return 0;
}

uid_t getuid(void) {
    libfake_log("VAGGA LIBFAKE: getuid, pretend we are root\n");
    return 0;
}

gid_t getgid(void) {
    libfake_log("VAGGA LIBFAKE: getgid, pretend we are root\n");
    return 0;
}

uid_t geteuid(void) {
    libfake_log("VAGGA LIBFAKE: geteuid, pretend we are root\n");
    return 0;
}

gid_t getegid(void) {
    libfake_log("VAGGA LIBFAKE: getegid, pretend we are root\n");
    return 0;
}

int execve(const char *filename, char *const argv[],
          char *const envp[]) {
    int (*orig_execve)(const char*, char*const argv[], char*const envp[]);
    orig_execve = dlsym(RTLD_NEXT, "execve");
    const char *base = strrchr(filename, '/');
    if(base && !strcmp(base, "/chroot")) {
        int nargs = 0;
        for(nargs = 0; argv[nargs]; ++nargs);
        char *newargv[nargs+6];
        newargv[0] = "vagga";
        newargv[1] = "_chroot";
        newargv[2] = "--writeable";
        newargv[3] = "--inventory";
        newargv[4] = "--environ=LD_PRELOAD=/tmp/inventory/libfake.so";
        memcpy(newargv+5, argv+1, nargs*sizeof(argv[0]));
        char* const *e;
        for(e = envp; *e; ++e) {
            if(!strncmp(*e, "LD_PRELOAD=", 11)) {
                (*e)[11] = 0;  // no ld-preload for vagga itself
            }
        }
        libfake_log("VAGGA LIBFAKE: replacing chroot\n");
        return (*orig_execve)(getenv("vagga_exe"), newargv, envp);
    } else {
        return (*orig_execve)(filename, argv, envp);
    }
}

int mount(const char *source, const char *target,
         const char *filesystemtype, unsigned long mountflags,
         const void *data) {
    libfake_log("VAGGA LIBFAKE: mount ignored\n");
    return 0;
}
