public class arg0 {
 public static int value;
 public static int pick(int value){return value+10;}
 public static int invoke(ShadowOther other){return arg0.pick(1);}
 public static int read(ShadowOther other){return arg0.value;}
 public static void write(ShadowOther other,int value){arg0.value=value;}
}
