import numpy as np
import matplotlib.pyplot as plt
import colorsys
import scipy.signal
from PIL import Image
import requests

def getURL(zoom,x,y):
  return "https://cyberjapandata.gsi.go.jp/xyz/std/"+str(zoom)+"/"+str(x)+"/"+str(y)+".png"

def clip(arr):
  mn = np.min(arr)
  mx = np.max(arr)
  if mn==mx:
    return np.zeros(arr.shape)
  else:
    return (arr-mn)/(mx-mn)

ids    = np.loadtxt("data/id.csv", delimiter = ",", dtype="int")
idzoom = ids[0] 
idx    = ids[1] 
idy    = ids[2] 
idsize = ids[3] 

system = np.loadtxt("data/system.csv", delimiter = ",", dtype="int")
data   = np.loadtxt("data/data.csv", delimiter = ",")
count  = np.loadtxt("data/count.csv", delimiter = ",")
light  = np.loadtxt("data/light.csv", delimiter = ",")
hmin = np.min(data)
hmax = np.max(data)
syss = np.max(system)
colors = []

imgrows = []
for y in range(idy-idsize,idy+idsize+1):
  print(str(y-(idy-idsize))+"/"+str(idsize*2+1),end="\r")
  imgs = []
  for x in range(idx-idsize,idx+idsize+1):
    im = np.array(Image.open(requests.get(getURL(idzoom,x,y), stream=True).raw).convert("RGB"))
    imgs.append(im)
  imgrows.append(np.hstack(imgs))
mapimg = np.vstack(imgrows)

for i in range(syss+1):
  colors.append(np.array(colorsys.hsv_to_rgb(np.random.rand(),0.5,1)))

pxs  = np.zeros((system.shape[0],system.shape[1],3))
pxs2 = np.zeros((system.shape[0],system.shape[1],3))
for y in range(system.shape[0]):
  if y%10 == 0:
    print(str(y)+"/"+str(system.shape[0]),end="\r")
  for x in range(system.shape[1]):
    #pxs[y][x]  = colors[system[y][x]] * ((data[y][x]-hmin) /(hmax-hmin) + 0.3)
    pxs2[y][x] = colors[system[y][x]]

#convolved = scipy.signal.convolve2d(count,np.ones((16,16))/(16**2),"valid")
#convolved = scipy.signal.convolve2d(count,np.ones((16,16))/(16**2),"valid")
root_count = np.power(count,0.2)
convolved  = scipy.signal.convolve2d(root_count,np.ones((4,4))/(4**2),"valid")
convolved2 = scipy.signal.convolve2d(root_count,np.ones((16,16))/(16**2),"valid")
pretty_count  = np.clip(root_count,0,np.max(convolved))
#pretty_count2 = np.clip(root_count,0,np.max(convolved2))
pxs2_255 = clip(pxs2)*255
mask = np.stack((clip(np.clip(root_count,1,np.max(convolved-1)+1)),)*3, axis=-1)
masked = pxs2_255*(mask)
result = np.uint8((mapimg)*(1-mask) + masked)
#print(root_count)

Image.fromarray(mapimg).save("data/map.png")
Image.fromarray(result).save("data/result.png")
Image.fromarray(np.uint8(pxs2_255)).save("data/systems.png")
Image.fromarray(np.uint8(clip(data)*255)).save("data/height.png")
Image.fromarray(np.uint8(clip(pretty_count)*255)).save("data/river.png")
Image.fromarray(np.uint8(clip(light)*255)).save("data/light.png")
Image.fromarray(np.uint8(masked)).save("data/colored.png")

plt.subplot(1,3,1)
plt.imshow(pxs2, interpolation='none')
plt.subplot(1,3,2)
#plt.imshow(np.log(count))
#plt.imshow(convolved)
plt.imshow(result)
plt.colorbar()
plt.subplot(1,3,3)
plt.imshow(data)
plt.colorbar()
plt.show()