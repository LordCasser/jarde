public class CH extends java.lang.Object {
    public CH() {
        super();
        return;
    }

    static java.lang.String reverse(java.lang.String arg0) {
        char[] local1;
        int local2;
        int local3;
        local1 = arg0.toCharArray();
        local2 = 0;
        local3 = local1.length - 1;
        while (local2 < local3) {
            char local4 = local1[local2];
            local1[local2] = local1[local3];
            local1[local3] = local4;
            local2 = local2 + 1;
            local3 = local3 - 1;
        }
        return new java.lang.String(local1);
    }

    static java.lang.String compress(java.lang.String arg0) {
        java.lang.StringBuilder local1;
        int local2;
        local1 = new java.lang.StringBuilder();
        local2 = 0;
        int local3;
        while (local2 < arg0.length()) {
            local3 = local2;
            while (local3 < arg0.length() && arg0.charAt(local3) == arg0.charAt(local2)) {
                local3 = local3 + 1;
            }
            if (local3 - local2 > 1) {
                local1.append(local3 - local2);
            }
            local1.append(arg0.charAt(local2));
            local2 = local3;
        }
        return local1.toString();
    }

    static boolean isPalindrome(java.lang.String arg0) {
        char[] local1;
        int local2;
        int local3;
        local1 = arg0.toCharArray();
        local2 = 0;
        local3 = local1.length - 1;
        while (local2 < local3) {
            if (local1[local2] != local1[local3]) {
                return false;
            } else {
                local2 = local2 + 1;
                local3 = local3 - 1;
            }
        }
        return true;
    }

    public static void main(java.lang.String[] arg0) {
        java.lang.System.out.println("" + reverse("hello") + "/" + compress("aaabcc") + "/" + isPalindrome("abba") + "/" + isPalindrome("abc"));
        return;
    }
}
