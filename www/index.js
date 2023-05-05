import * as wasm from 'tradecharts';
var wglchart;

var app = new Vue({
    el: '#app',
    data: {
        activeticker: "GAZP",
        issuers: [
          {ticker: "GAZP", name: "Газпром" },
          {ticker: "NVTK", name: "Новатэк" },
          {ticker: "ROSN", name: "Роснефть" },
          {ticker: "LKOH", name: "Лукойл" },
          {ticker: "TATN", name: "Татнефть (об)" },
          {ticker: "TATNP", name: "Татнефть (пр)" },
          {ticker: "SIBN", name: "Газпромнефть" },
          {ticker: "IRAO", name: "ИнтерРАО" },
          {ticker: "PHOR", name: "Фосагро" },
          {ticker: "NLMK", name: "НЛМК" },
          {ticker: "MAGN", name: "ММК" },
          {ticker: "GMKN", name: "Норникель" },
          {ticker: "RUAL", name: "Русал" },
          {ticker: "PLZL", name: "Полюс Золото" },
          {ticker: "POLY", name: "Полиметалл" },
          {ticker: "ALRS", name: "Алроса" },
          {ticker: "SBER", name: "Сбербанк (об)" },
          {ticker: "SBERP", name: "Сбербанк (пр)" },
          {ticker: "VTBR", name: "ВТБ" },
          {ticker: "AFKS", name: "АФК Система" },
          {ticker: "MOEX", name: "Московская биржа" },
          {ticker: "YNDX", name: "Яндекс" },
          {ticker: "VKCO", name: "ВК" },
          {ticker: "RTKM", name: "Ростелеком" },
          {ticker: "MTSS", name: "МТС" },
          {ticker: "FIVE", name: "Х5" },
          {ticker: "MGNT", name: "Магнит" },
          {ticker: "BELU", name: "Белуга" },
          {ticker: "SGZH", name: "Сегежа" },
        ],
    },
    mounted() {
      // console.log(this.tickers[this.counter % this.tickers.length]);
      this.adjustResizing();

      wglchart = wasm.TradeChart.new();
      wglchart.display(this.activeticker);

      window.addEventListener('resize', this.onWindowResize);

      // var canvas = document.getElementById('axe');
      // var ctx = canvas.getContext('2d');
      // // ctx.fillRect(25,25,100,100);
      // // ctx.clearRect(45,45,60,60);
      // // ctx.strokeRect(50,50,50,50);
      // ctx.fillStyle = "#111";
      // ctx.strokeStyle = "#F00";
      // ctx.font = "10pt Tahoma";
      // ctx.fillText("20.04", 12, 12);
    },
    methods: {
      adjustResizing () {
        const chart = document.getElementById("chart");
        chart.width = window.innerWidth;
        chart.height = window.innerHeight;
      },
      onWindowResize (e) {
        this.adjustResizing();
        wglchart.draw();
      },
      showChart (ticker) {
        this.activeticker = ticker;
        // console.log(ticker);
        wglchart.display(this.activeticker);
      },
      shiftChart (b) {
        wglchart.shift(b ? 1.0 : -1.0);
      },
    }
  })

  