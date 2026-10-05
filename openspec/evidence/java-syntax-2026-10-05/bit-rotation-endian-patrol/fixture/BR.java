public class BR {
    static final char[] HEX = "0123456789abcdef".toCharArray();          // 查表（clinit toCharArray）
    static String toHex(int v){ StringBuilder sb = new StringBuilder(8); for(int i = 7; i >= 0; i--){ sb.append(HEX[(v >>> (i * 4)) & 0xF]); } return sb.toString(); }   // 移位+掩码查表
    static int rotl(int v, int d){ return Integer.rotateLeft(v, d); }      // intrinsics 候选
    static int revBytes(int v){ return Integer.reverseBytes(v); }
    static int bitCount(int v){ return Integer.bitCount(v); }
    static String bytes(byte[] bs){ StringBuilder sb = new StringBuilder(); for(byte b : bs){ sb.append(Character.forDigit((b >> 4) & 0xF, 16)).append(Character.forDigit(b & 0xF, 16)); } return sb.toString(); }  // byte 序列化 hex
    public static void main(String[] a){ System.out.println(""+toHex(0xDEADBEEF)+"/"+rotl(0x00FF00FF, 8)+"/"+revBytes(0x11223344)+"/"+bitCount(0xFF00FF0F)+"/"+bytes(new byte[]{(byte)0xCA, (byte)0xFE})); }
}
