public class PS {                                             // 词法扫描器形状：for+switch+SB+计数
    static String scan(String s){
        StringBuilder out = new StringBuilder();
        int words = 0; int digits = 0; char prev = 0;
        for(int i = 0; i < s.length(); i++){
            char c = s.charAt(i);
            switch(c){
                case ' ': case '\t': break;                   // 跳过空白
                case '0': case '1': case '2': case '3': digits++; break;
                default:
                    if(c == prev){ continue; }                // 连续重复跳过
                    out.append(Character.toUpperCase(c));
                    words++;
            }
            prev = c;
        }
        return out.toString() + "/" + words + "/" + digits;
    }
    static int sumSquares(int[] xs){ int t = 0; for(int x : xs){ t += x * x; } return t; }   // 循环乘积累积
    public static void main(String[] a){ System.out.println(scan("a b\t112 a z")+"/"+sumSquares(new int[]{2,3})); }
}
