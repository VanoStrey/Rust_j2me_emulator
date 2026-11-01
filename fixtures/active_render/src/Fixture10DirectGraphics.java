package fixtures;

import javax.microedition.midlet.MIDlet;
import javax.microedition.lcdui.Display;
import javax.microedition.lcdui.Graphics;
import javax.microedition.lcdui.game.GameCanvas;
import com.nokia.mid.ui.DirectGraphics;
import com.nokia.mid.ui.DirectUtils;

public class Fixture10DirectGraphics extends MIDlet {
    public static boolean npeThrown = false;
    public static boolean iaeThrown = false;
    public static boolean aioobeThrown = false;
    public static int readbackPixel = 0;

    public static class DgCanvas extends GameCanvas {
        public DgCanvas() { super(false); }

        public void testContracts() {
            Graphics g = getGraphics();
            DirectGraphics dg = DirectUtils.getDirectGraphics(g);

            // 1. Draw pixels with format ARGB 8888
            int[] src = new int[] { 0xFFFF0000, 0xFF00FF00, 0xFF0000FF, 0xFFFFFF00 };
            dg.drawPixels(src, true, 0, 2, 10, 10, 2, 2, 0, DirectGraphics.TYPE_INT_8888_ARGB);

            // 2. Readback with getPixels
            int[] dst = new int[4];
            dg.getPixels(dst, 0, 2, 10, 10, 2, 2, DirectGraphics.TYPE_INT_8888_ARGB);
            readbackPixel = dst[0]; // should be 0xFFFF0000

            // 3. Exception testing: null array -> NPE
            try {
                dg.drawPixels((int[])null, true, 0, 2, 0, 0, 2, 2, 0, 1);
            } catch (NullPointerException e) {
                npeThrown = true;
            }

            // 4. Bad manipulation -> IAE
            try {
                dg.drawPixels(src, true, 0, 2, 0, 0, 2, 2, 45, 1); // 45 is invalid rot
            } catch (IllegalArgumentException e) {
                iaeThrown = true;
            }

            // 5. Negative offset -> AIOOBE
            try {
                dg.drawPixels(src, true, -1, 2, 0, 0, 2, 2, 0, 1);
            } catch (ArrayIndexOutOfBoundsException e) {
                aioobeThrown = true;
            }

            flushGraphics();
        }
    }

    protected void startApp() {
        DgCanvas c = new DgCanvas();
        Display.getDisplay(this).setCurrent(c);
        c.testContracts();
    }
    protected void pauseApp() {}
    protected void destroyApp(boolean u) {}
}
