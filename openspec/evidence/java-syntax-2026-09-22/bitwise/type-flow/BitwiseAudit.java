public class BitwiseAudit {
 public static boolean nested(boolean a,boolean b,boolean c){return a & (b ^ (c | true));}
 public static boolean copied(boolean a,boolean b,boolean c){boolean first=a|b;boolean copy=first;boolean last=copy&c;return last;}
 public static boolean hoisted(boolean a,boolean b,boolean c){boolean x=a&b;if(c)x=a|b;return x;}
 public static int passed(boolean a,boolean b,boolean c){return BitwiseEffects.accept((a ^ b) & c);}
 public static boolean array(boolean[] a){return a[0]^a[1];}
 public static int promoted(byte a,char b){return a|b;}
}
