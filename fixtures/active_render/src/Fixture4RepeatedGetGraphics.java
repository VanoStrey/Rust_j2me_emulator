package fixtures;

import javax.microedition.midlet.MIDlet;
import javax.microedition.lcdui.Display;
import javax.microedition.lcdui.Graphics;
import javax.microedition.lcdui.game.GameCanvas;

public class Fixture4RepeatedGetGraphics extends MIDlet {
    public static class TestCanvas extends GameCanvas {
        public TestCanvas() {
            super(false);
        }
        public void runTest() {
            Graphics g1 = getGraphics();
            g1.setColor(0x000000FF); // Blue
            g1.fillRect(10, 10, 20, 20);
            g1.translate(50, 50);
            g1.setClip(5, 5, 10, 10);

            // Call getGraphics() again
            Graphics g2 = getGraphics();
            // Contract check: g2 translation must be 0,0 and clip must be reset to full screen
            boolean resetOk = (g2.getTranslateX() == 0 && g2.getTranslateY() == 0 &&
                               g2.getClipX() == 0 && g2.getClipY() == 0 &&
                               g2.getClipWidth() == 240 && g2.getClipHeight() == 320);
            if (resetOk) {
                // Draw second box at (40, 40, 20, 20) in Green
                g2.setColor(0x0000FF00);
                g2.fillRect(40, 40, 20, 20);
            }
            flushGraphics();
        }
    }

    protected void startApp() {
        TestCanvas c = new TestCanvas();
        Display.getDisplay(this).setCurrent(c);
        c.runTest();
    }
    protected void pauseApp() {}
    protected void destroyApp(boolean u) {}
}
