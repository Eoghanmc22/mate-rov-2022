Make sure ffmpeg is installed

Install YOLOv5 (only needed once)

```bash
git clone https://github.com/ultralytics/yolov5  # clone
cd yolov5
pip install -r requirements.txt  # install
cd ../
```

Place video in directory

Reduce framerate and run ai

```bash
ffmpeg -i video.mp4 -r 10 out.mp4 -y && python3 yolov5/detect.py --source out.mp4 --weights model/best.pt --data model/data.yaml
```

Video will be placed in yolov5/runs/detect/exp*/out.mp4