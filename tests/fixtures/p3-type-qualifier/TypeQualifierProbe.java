public class TypeQualifierProbe {
 public static int ownPick(int value){return value+50;}
 public static int ownCall(){return ownPick(1);}
 public static int invoke(ShadowOther other){return arg0.pick(1)+arg0_2.pick(1);}
 public static int read(ShadowOther other){return arg0.value+arg0_2.value;}
 public static void write(ShadowOther other,int value){arg0.value=value;arg0_2.value=value+1;}
}
