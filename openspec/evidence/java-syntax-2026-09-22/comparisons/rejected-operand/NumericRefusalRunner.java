public class NumericRefusalRunner {
 public static void main(String[] args){
  for(float x:new float[]{Float.NaN,-1f,-0.0f,0.0f,1f}){NumericRefusalEffects.calls=0;System.out.println(NumericRefusal.value(x)+":"+NumericRefusalEffects.calls);}
  NumericRefusalEffects.calls=0;NumericRefusalEffects.failing=true;
  try{NumericRefusal.value(1f);System.out.println("returned");}catch(RuntimeException e){System.out.println(e.getClass().getName()+":"+NumericRefusalEffects.calls);}
 }
}
