import urllib.request
#import numpy as np
#import matplotlib.pyplot as plt

#lat  =  35.000000
#lon  = 137.369785
#zoom = 11

def getId(zoom, lat, lon):
  # 緯度･経度をタイル番号に変換
  # https://docs.microsoft.com/ja-jp/azure/azure-maps/zoom-levels-and-tile-grid
  # https://www.trail-note.net/tech/coordinate/
  L = 85.05112878
  x = int(2**(zoom-1)*(lon/180+1))
  y = int(2**(zoom-1)/np.pi*(np.arctanh(np.sin(L/180*np.pi))-np.arctanh(np.sin(lat/180*np.pi))))
  return (x,y)

def idToHeightmap(zoom,x,y):
  raw = np.genfromtxt(urllib.request.urlopen("https://cyberjapandata.gsi.go.jp/xyz/dem/"+str(zoom)+"/"+str(x)+"/"+str(y)+".txt"), delimiter=",", missing_values="e", filling_values=0)
  return raw.astype(np.float64)

def showFFT2(zoom, lat, lon):
  (x,y) = getId(zoom,lat,lon)
  print("https://cyberjapandata.gsi.go.jp/xyz/std/"+str(zoom)+"/"+str(x)+"/"+str(y)+".png")
  print("https://cyberjapandata.gsi.go.jp/xyz/dem/"+str(zoom)+"/"+str(x)+"/"+str(y)+".txt")
  print("https://maps.gsi.go.jp/#"+str(zoom)+"/"+str(lat)+"/"+str(lon)+"/")
  heightmap = idToHeightmap(zoom,x,y)
  fig = plt.figure()
  fig.add_subplot(1, 3, 1)
  plt.imshow(heightmap)
  fig.add_subplot(1, 3, 2)
  plt.imshow(np.log(np.absolute(np.fft.fftshift(np.fft.fft2(heightmap)))), 'gray', vmin = -3, vmax = 19)
  #fig.add_subplot(1, 3, 3)
  #plt.imshow(np.real(np.fft.ifft2(np.fft.fft2(heightmap))))
  print(np.max(np.log(np.absolute(np.fft.fftshift(np.fft.fft2(heightmap))))), np.min(np.log(np.absolute(np.fft.fftshift(np.fft.fft2(heightmap))))))

showFFT2(14, 35.000000, 137.369785)
showFFT2(14, 35.726169, 139.150085)
showFFT2(14, 35.163687, 137.680492)
showFFT2(14, 35.659877, 139.677601)
showFFT2(14, 35.163687, 137.680492)