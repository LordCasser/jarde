public class ThrowSwitch {public static void select(int x,RuntimeException a,RuntimeException b){switch(x){case 0:throw a;case 1:throw b;default:throw null;}}}
