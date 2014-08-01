#define _GNU_SOURCE
#include <dlfcn.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <sys/auxv.h>
#include <fcntl.h>
#include <unistd.h>
#include <fcntl.h>
#include <sys/stat.h>
#include <string.h>

inline static void libfake_log(const char *val) {
    write(2, val, strlen(val));
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
    const char *execfn = (const char *)getauxval(AT_EXECFN);
    const char *base = strrchr(execfn, '/');
    if(base ? !strcmp(base, "/vagga") : !strcmp(execfn, "vagga")) {
        libfake_log("VAGGA LIBFAKE: getuid, is real for vagga\n");
        return getauxval(AT_UID);
    }
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
    int (*original_execve)(const char*, char*const argv[], char*const envp[]);
    original_execve = dlsym(RTLD_NEXT, "execve");
    const char *base = strrchr(filename, '/');
    if(base && !strcmp(base, "/chroot")) {
        int nargs = 0;
        for(nargs = 0; argv[nargs]; ++nargs);
        char *newargv[nargs+3];
        newargv[0] = "vagga";
        newargv[1] = "_chroot";
        newargv[2] = "--writeable";
        memcpy(newargv+3, argv+1, nargs*sizeof(argv[0]));
        libfake_log("VAGGA LIBFAKE: replacing chroot\n");
        return (*original_execve)((const char *)getenv("vagga_exe"), newargv, envp);
    } else {
        return (*original_execve)(filename, argv, envp);
    }
}

