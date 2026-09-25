public class Var {
    static int many(int first, int... rest) {
        return first;
    }

    static byte[] merge(byte[]... parts) {
        return parts[0];
    }

    static int plain(int[] xs) {
        return xs.length;
    }
}
