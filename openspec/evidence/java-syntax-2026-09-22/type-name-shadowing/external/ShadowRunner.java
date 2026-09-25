public class ShadowRunner {
 public static void main(String[]args){for(ShadowOther other:new ShadowOther[]{null,new ShadowOther()}){
  String label=other==null?"null":"object";arg0.value=3;ShadowOther.value=7;
  System.out.println(label+":invoke:"+ShadowExternal.invoke(other));
  System.out.println(label+":read:"+ShadowExternal.read(other));
  ShadowExternal.write(other,9);System.out.println(label+":write:"+arg0.value+":"+ShadowOther.value);
 }}
}
