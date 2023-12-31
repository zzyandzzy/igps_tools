# igps_tools

[中文](./README_zh.md)

`igps_tools` is a series of easy to operate iGPS tools.

- [workout](https://github.com/zzyandzzy/igps_tools/releases) Import the training plan
  from [icu](https://intervals.icu/) into iGPS

  Use tutorial
  - Download and extract the training plan as a FIT file from [icu](https://intervals.icu/)
    ![img.png](images/img.png)
  - Download [workout](https://github.com/zzyandzzy/igps_tools/releases) binary files
  - Fill in the TOKEN (or username/password) and execute the workout binary as shown below:

```shell
# Use iGPS token
./workout --fit-zip ./fit.zip --token "Your iGPS token"
# Use iGPS username/password
./workout --fit-zip ./fit.zip --username "Your iGPS username" --password "Your iGPS password"
# increment power 10w
./workout --fit-zip ./fit.zip --token "Your iGPS token" add power -v 10
# increment duration 10s
./workout --fit-zip ./fit.zip --token "Your iGPS token" add duration -v 10
# For more details of the order, please check
./workout --help
```
- [xingzhe](https://github.com/zzyandzzy/igps_tools/releases) Convert xingzhe history data to fit.

```shell
# Download 202305 data convert fit
./xingzhe -y 2023 -m 5 -u uid -c 'cookie'
```