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
# For more details of the order, please check
./workout --help
```
