import { jsPDF } from "jspdf";
import autoTable from "jspdf-autotable";

export interface AttendanceEntry {
  date: string;
  time: string;
  name: string;
  course: string;
  purpose: string;
}

export const usePdfGenerator = () => {
  const generatePDF = async (
    attendanceData: AttendanceEntry[],
    courses: string[],
    selectedDate: string,
    schoolYear: string,
    logoBase64: string,
    toast: any,
    formattedSelectedDateForTitle: string
  ) => {
    const pdf = new jsPDF({
      orientation: "landscape",
      unit: "in",
      format: [13, 8.5],
    });

    const pageHeight = pdf.internal.pageSize.getHeight();
    const pageWidth = pdf.internal.pageSize.getWidth();
    const margin = 0.5;

    // Add header function
    const addHeader = (pdfInstance: jsPDF) => {
      const logoWidth = 0.6;
      const logoHeight = 0.6;
      const logoX = margin;
      const logoY = margin / 2;
      const textStartX = logoX + logoWidth + 0.1;

      // College information constants
      const collegeName = "General de Jesus College";
      const address = "VALLARTA ST. POBLACION, SAN ISIDRO, NUEVA ECIJA";
      const contactInfo =
        "(+6344) 940-6161 | gdjcdejesus@gmail.com | gdjcdejesus.edu.ph";

      // Draw Logo
      pdfInstance.addImage(
        logoBase64,
        "PNG",
        logoX,
        logoY,
        logoWidth,
        logoHeight
      );

      // Draw Header Text
      pdfInstance.setFontSize(14).setFont(undefined, "bold");
      pdfInstance.text(collegeName, textStartX, logoY + 0.15);

      pdfInstance.setFontSize(8).setFont(undefined, "normal");
      pdfInstance.text(address, textStartX, logoY + 0.3);
      pdfInstance.text(contactInfo, textStartX, logoY + 0.45);

      // Report Title Line
      pdfInstance.setFontSize(10).setFont(undefined, "bold");
      const reportTitle = `Daily Record of Library Users SY: ${schoolYear} ${formattedSelectedDateForTitle}`;
      const titleWidth = pdfInstance.getTextWidth(reportTitle);
      pdfInstance.text(
        reportTitle,
        (pageWidth - titleWidth) / 2,
        logoY + logoHeight + 0.15
      );
    };

    // Add footer function
    const addFooter = (
      pdfInstance: jsPDF,
      pageNumber: number,
      totalPages: number
    ) => {
      pdfInstance.setFontSize(8).setFont(undefined, "normal");
      const footerText = `Page ${pageNumber} of ${totalPages}`;
      const textWidth = pdfInstance.getTextWidth(footerText);
      pdfInstance.text(
        footerText,
        pageWidth - margin - textWidth,
        pageHeight - margin / 2
      );
    };

    // Helper functions for the PDF
    const getTotalForCourse = (course: string) => {
      return attendanceData.filter((entry) => entry.course === course).length;
    };

    const getTotalForPurpose = (purpose: string) => {
      const uniquePurposes = Array.from(
        new Set(attendanceData.map((entry) => entry.purpose))
      ).sort();
      return attendanceData.filter((entry) => entry.purpose === purpose).length;
    };

    const getChunkedCourses = (courseList: string[]) => {
      const chunkSize = 5;
      const result = [];
      for (let i = 0; i < courseList.length; i += chunkSize) {
        result.push(courseList.slice(i, i + chunkSize));
      }
      return result;
    };

    const chunkedCourses = getChunkedCourses(courses);
    const uniquePurposes = Array.from(
      new Set(attendanceData.map((entry) => entry.purpose))
    ).sort();

    // Main table data preparation
    const head = [
      {
        content: "DATE",
        styles: { halign: "center", fontStyle: "bold", fontSize: 7 },
      },
      {
        content: "TIME",
        styles: { halign: "center", fontStyle: "bold", fontSize: 7 },
      },
      {
        content: "NAME (Last Name, First Name)",
        styles: { halign: "left", fontStyle: "bold", fontSize: 7 },
      },
      // Dynamically add course headers
      ...courses.map((course) => ({
        content: course,
        styles: {
          halign: "center",
          fontStyle: "bold",
          fontSize: 6,
          cellWidth: 0.3,
        },
      })),
      {
        content: "Purpose of Visit",
        styles: { halign: "left", fontStyle: "bold", fontSize: 7 },
      },
    ];

    const body = attendanceData.map((entry) => [
      entry.date,
      entry.time,
      { content: entry.name, styles: { halign: "left", fontSize: 7 } },
      ...courses.map((course) => (entry.course === course ? "✓" : "")),
      { content: entry.purpose, styles: { halign: "left", fontSize: 7 } },
    ]);

    // Add total row
    const totalRow = [
      {
        content: "Total:",
        colSpan: 3,
        styles: { halign: "right", fontStyle: "bold", fontSize: 7 },
      },
      ...courses.map((course) => ({
        content: getTotalForCourse(course).toString(),
        styles: { halign: "center", fontStyle: "bold", fontSize: 7 },
      })),
      {
        content: `Grand Total: ${attendanceData.length}`,
        styles: { halign: "right", fontStyle: "bold", fontSize: 7 },
      },
    ];
    body.push(totalRow);

    let finalY = 0;
    let currentPage = 0;

    // Generate main table
    autoTable(pdf, {
      head: [head],
      body: body,
      startY: 1.5,
      margin: { top: 1.5, right: margin, bottom: margin, left: margin },
      theme: "grid",
      styles: {
        fontSize: 7,
        cellPadding: 0.02,
        overflow: "linebreak",
      },
      headStyles: {
        fillColor: [230, 230, 230],
        textColor: [0, 0, 0],
        fontStyle: "bold",
        lineWidth: 0.01,
        lineColor: [0, 0, 0],
      },
      bodyStyles: {
        lineWidth: 0.01,
        lineColor: [0, 0, 0],
      },
      columnStyles: {
        2: { cellWidth: 2.5, halign: "left" },
        ...Object.fromEntries(
          courses.map((_, i) => [
            3 + i,
            { cellWidth: 0.3, halign: "center", fontSize: 8 },
          ])
        ),
        [3 + courses.length]: { cellWidth: "auto", halign: "left" },
      },
      didDrawPage: (data) => {
        currentPage = data.pageNumber;
        addHeader(pdf);
        finalY = data.cursor?.y ?? pageHeight - margin;
      },
      willDrawCell: (data) => {
        if (data.row.index === body.length - 1) {
          data.cell.styles.fontStyle = "bold";
          data.cell.styles.fillColor = [240, 240, 240];
        }
      },
    });

    // Summary section
    const summaryStartY = finalY + 0.3;
    const summarySectionHeightEstimate = 2.5;

    if (
      currentPage === 1 &&
      summaryStartY + summarySectionHeightEstimate > pageHeight - margin
    ) {
      pdf.addPage();
      currentPage++;
      addHeader(pdf);
      finalY = 1.5;
    } else {
      finalY = summaryStartY;
    }

    const summaryStartX = margin;
    const summaryWidth = pageWidth - 2 * margin;

    // Summary Header
    pdf.setFontSize(12).setFont(undefined, "bold");
    pdf.setFillColor(204, 204, 204);
    pdf.rect(summaryStartX, finalY, summaryWidth, 0.3, "F");
    pdf.setTextColor(0, 0, 0);
    const summaryHeaderText = `Total Attendance by Purpose - ${formattedSelectedDateForTitle}`;
    const summaryHeaderWidth = pdf.getTextWidth(summaryHeaderText);
    pdf.text(
      summaryHeaderText,
      (pageWidth - summaryHeaderWidth) / 2,
      finalY + 0.2
    );
    finalY += 0.4;

    // Course Summary Table
    const courseSummaryBody = [];
    for (const chunk of chunkedCourses) {
      const row: any[] = [];
      chunk.forEach((course) => {
        row.push(
          {
            content: `${course}:`,
            styles: { fontStyle: "bold", halign: "left" },
          },
          {
            content: getTotalForCourse(course).toString(),
            styles: { halign: "left" },
          }
        );
      });
      // Fill empty slots if needed
      const cellsInRow = row.length;
      const targetCells = 10; // 5 courses * 2 cells each
      if (cellsInRow < targetCells) {
        row.push({
          content: "",
          colSpan: targetCells - cellsInRow,
          styles: { fillColor: [255, 255, 255] },
        });
      }
      courseSummaryBody.push(row);
    }
    // Add Total Attendance row
    courseSummaryBody.push([
      {
        content: `Total Attendance: ${attendanceData.length}`,
        colSpan: 10,
        styles: { halign: "right", fontStyle: "bold", fontSize: 9 },
      },
    ]);

    autoTable(pdf, {
      body: courseSummaryBody,
      startY: finalY,
      theme: "plain",
      margin: { left: margin, right: margin },
      tableWidth: summaryWidth,
      styles: { fontSize: 8, cellPadding: 0.05 },
      columnStyles: {
        0: { cellWidth: 0.8 },
        1: { cellWidth: 0.5 },
        2: { cellWidth: 0.8 },
        3: { cellWidth: 0.5 },
        4: { cellWidth: 0.8 },
        5: { cellWidth: 0.5 },
        6: { cellWidth: 0.8 },
        7: { cellWidth: 0.5 },
        8: { cellWidth: 0.8 },
        9: { cellWidth: 0.5 },
      },
      willDrawCell: (data) => {
        if (data.row.index === courseSummaryBody.length - 1) {
          pdf.setDrawColor(0, 0, 0);
          pdf.setLineWidth(0.015);
          pdf.line(
            data.cell.x,
            data.cell.y,
            data.cell.x + data.cell.width,
            data.cell.y
          );
        }
      },
      didDrawPage: (data) => {
        addHeader(pdf);
        currentPage = data.pageNumber;
        finalY = data.cursor?.y ?? pageHeight - margin;
      },
      didDrawTable: (data) => {
        finalY = data.cursor.y + 0.1;
      },
    });

    // Purpose Summary Table
    const purposeHead = [
      uniquePurposes.map((p) => ({
        content: p.toUpperCase(),
        styles: { halign: "center", fontStyle: "bold", fontSize: 8 },
      })),
    ];
    const purposeBody = [
      uniquePurposes.map((p) => ({
        content: getTotalForPurpose(p).toString(),
        styles: { halign: "center", fontSize: 8 },
      })),
    ];

    autoTable(pdf, {
      head: purposeHead,
      body: purposeBody,
      startY: finalY + 0.2,
      theme: "grid",
      margin: { left: margin * 4, right: margin * 4 },
      styles: { fontSize: 8, cellPadding: 0.05 },
      headStyles: {
        fillColor: [230, 230, 230],
        textColor: [0, 0, 0],
        fontStyle: "bold",
      },
      didDrawPage: (data) => {
        addHeader(pdf);
        currentPage = data.pageNumber;
      },
    });

    // Add footers to all pages
    const totalPages = (pdf as any).internal.getNumberOfPages();
    for (let i = 1; i <= totalPages; i++) {
      pdf.setPage(i);
      addFooter(pdf, i, totalPages);
    }

    return pdf;
  };

  return { generatePDF };
};
