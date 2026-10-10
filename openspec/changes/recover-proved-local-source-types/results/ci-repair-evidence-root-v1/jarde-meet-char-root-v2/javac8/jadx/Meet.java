package defpackage;

import java.util.List;

/* JADX INFO: loaded from: input.jar:Meet.class */
public class Meet {
    static int at(String str, int i) {
        return str.charAt(i);
    }

    static int viaStore(char c) {
        return c;
    }

    static int pass(int i) {
        return i;
    }

    static int fieldArg(String str) {
        return pass(str.charAt(0));
    }

    static long viaStoreLong(int i) {
        return i;
    }

    static byte trunc(int i) {
        return (byte) i;
    }

    static char grade(int i) {
        switch (i) {
            case 80:
                return 'B';
            case 90:
            case 95:
                return 'A';
            default:
                return 'C';
        }
    }

    static int stat() {
        return 7;
    }

    static int viaRef(Meet meet) {
        return stat();
    }

    static void unchecked(List list) {
        list.add("x");
    }

    static void pop2Control() {
        System.nanoTime();
    }
}
