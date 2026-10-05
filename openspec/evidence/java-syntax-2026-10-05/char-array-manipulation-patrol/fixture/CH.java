public class CH {
    static String reverse(String s){                                  // toCharArray + 双指针 swap + new String
        char[] cs = s.toCharArray();
        for(int i = 0, j = cs.length - 1; i < j; i++, j--){ char t = cs[i]; cs[i] = cs[j]; cs[j] = t; }
        return new String(cs);
    }
    static String compress(String s){                                 // run-length：双指针读 + SB 写
        StringBuilder sb = new StringBuilder();
        int i = 0;
        while(i < s.length()){
            int j = i;
            while(j < s.length() && s.charAt(j) == s.charAt(i)){ j++; }
            if(j - i > 1){ sb.append(j - i); }
            sb.append(s.charAt(i));
            i = j;
        }
        return sb.toString();
    }
    static boolean isPalindrome(String s){                            // 双指针布尔早退
        char[] cs = s.toCharArray();
        for(int i = 0, j = cs.length - 1; i < j; i++, j--){ if(cs[i] != cs[j]){ return false; } }
        return true;
    }
    public static void main(String[] a){ System.out.println(""+reverse("hello")+"/"+compress("aaabcc")+"/"+isPalindrome("abba")+"/"+isPalindrome("abc")); }
}
